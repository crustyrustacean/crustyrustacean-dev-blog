// src/lib/repositories/category_pg.rs

//! PostgreSQL implementation of the CategoryRepository trait.

use async_trait::async_trait;
use slug::slugify;
use uuid::Uuid;

use super::category::{
    CategoryRecord, CategoryRepository, CategoryWithCount, NewCategory, UpdateCategoryData,
};
use super::error::{RepoResult, RepositoryError};

/// PostgreSQL implementation of the CategoryRepository.
#[derive(Debug, Clone)]
pub struct PgCategoryRepository {
    db: sqlx::PgPool,
}

impl PgCategoryRepository {
    /// Create a new PostgreSQL category repository.
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CategoryRepository for PgCategoryRepository {
    async fn create(&self, category: &NewCategory) -> RepoResult<CategoryRecord> {
        let record = sqlx::query_as::<_, (Uuid, String, String, Option<String>)>(
            r"INSERT INTO categories (name, slug, description)
              VALUES ($1, $2, $3)
              RETURNING id, name, slug, description",
        )
        .bind(&category.name)
        .bind(slugify(&category.name))
        .bind(&category.description)
        .fetch_one(&self.db)
        .await
        .map(|(id, name, slug, description)| CategoryRecord {
            id,
            name,
            slug,
            description,
        })?;

        Ok(record)
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<CategoryRecord>> {
        let record = sqlx::query_as::<_, (Uuid, String, String, Option<String>)>(
            "SELECT id, name, slug, description FROM categories WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .map(|(id, name, slug, description)| CategoryRecord {
            id,
            name,
            slug,
            description,
        });

        Ok(record)
    }

    async fn find_by_slug(&self, slug: &str) -> RepoResult<Option<CategoryRecord>> {
        let record = sqlx::query_as::<_, (Uuid, String, String, Option<String>)>(
            "SELECT id, name, slug, description FROM categories WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(&self.db)
        .await?
        .map(|(id, name, slug, description)| CategoryRecord {
            id,
            name,
            slug,
            description,
        });

        Ok(record)
    }

    async fn find_by_slug_with_count(
        &self,
        slug: &str,
    ) -> RepoResult<Option<CategoryWithCount>> {
        let record = sqlx::query_as::<_, (Uuid, String, String, Option<String>, i64)>(
            r"SELECT c.id, c.name, c.slug, c.description,
                     (SELECT COUNT(*) FROM articles a
                      WHERE a.category_id = c.id AND a.draft = FALSE) as article_count
              FROM categories c
              WHERE c.slug = $1",
        )
        .bind(slug)
        .fetch_optional(&self.db)
        .await?
        .map(|(id, name, slug, description, article_count)| CategoryWithCount {
            category: CategoryRecord {
                id,
                name,
                slug,
                description,
            },
            article_count,
        });

        Ok(record)
    }

    async fn find_by_name(&self, name: &str) -> RepoResult<Option<CategoryRecord>> {
        let record = sqlx::query_as::<_, (Uuid, String, String, Option<String>)>(
            "SELECT id, name, slug, description FROM categories WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.db)
        .await?
        .map(|(id, name, slug, description)| CategoryRecord {
            id,
            name,
            slug,
            description,
        });

        Ok(record)
    }

    async fn list_with_counts(&self) -> RepoResult<Vec<CategoryWithCount>> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, Option<String>, i64)>(
            r"SELECT c.id, c.name, c.slug, c.description,
                     (SELECT COUNT(*) FROM articles a
                      WHERE a.category_id = c.id AND a.draft = FALSE) as article_count
              FROM categories c
              ORDER BY c.name",
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, name, slug, description, article_count)| CategoryWithCount {
                category: CategoryRecord {
                    id,
                    name,
                    slug,
                    description,
                },
                article_count,
            })
            .collect())
    }

    async fn update(&self, slug: &str, data: &UpdateCategoryData) -> RepoResult<CategoryRecord> {
        let mut updates = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some(name) = &data.name {
            updates.push(format!("name = ${}", binds.len() + 1));
            binds.push(name.clone());
        }
        if let Some(description) = &data.description {
            updates.push(format!("description = ${}", binds.len() + 1));
            binds.push(description.clone());
        }

        // Recompute the slug when the name changes.
        if data.name.is_some() {
            let name = data.name.as_deref().unwrap_or_default();
            updates.push(format!("slug = ${}", binds.len() + 1));
            binds.push(slugify(name));
        }

        if updates.is_empty() {
            return self
                .find_by_slug(slug)
                .await?
                .ok_or_else(|| RepositoryError::NotFound(format!("Category with slug {}", slug)));
        }

        updates.push("updated_at = NOW()".to_string());
        let slug_bind = binds.len() + 1;
        binds.push(slug.to_string());

        let query_str = format!(
            "UPDATE categories SET {} WHERE slug = ${} \
             RETURNING id, name, slug, description",
            updates.join(", "),
            slug_bind
        );

        let mut query = sqlx::query_as::<_, (Uuid, String, String, Option<String>)>(&query_str);
        for value in &binds {
            query = query.bind(value);
        }

        let record = query.fetch_optional(&self.db).await.map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                RepositoryError::NotFound(format!("Category with slug {}", slug))
            } else {
                e.into()
            }
        })?;

        match record {
            Some((id, name, slug, description)) => Ok(CategoryRecord {
                id,
                name,
                slug,
                description,
            }),
            None => Err(RepositoryError::NotFound(format!(
                "Category with slug {}",
                slug
            ))),
        }
    }

    async fn delete(&self, slug: &str) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM categories WHERE slug = $1")
            .bind(slug)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Category with slug {}",
                slug
            )));
        }

        Ok(())
    }
}
