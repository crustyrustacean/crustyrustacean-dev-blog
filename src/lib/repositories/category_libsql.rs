// src/lib/repositories/category_libsql.rs

//! LibSQL implementation of the CategoryRepository trait.

use async_trait::async_trait;
use chrono::Utc;
use slug::slugify;
use uuid::Uuid;

use crate::database::DatabaseConnection;

use super::category::{
    CategoryRecord, CategoryRepository, CategoryWithCount, NewCategory, UpdateCategoryData,
};
use super::error::{RepoResult, RepositoryError};

/// LibSQL implementation of the CategoryRepository.
#[derive(Debug, Clone)]
pub struct LibSqlCategoryRepository {
    db: DatabaseConnection,
}

impl LibSqlCategoryRepository {
    /// Create a new LibSQL category repository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CategoryRepository for LibSqlCategoryRepository {
    async fn create(&self, category: &NewCategory) -> RepoResult<CategoryRecord> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let id = Uuid::new_v4();
        let slug = slugify(&category.name);
        let now = Utc::now();

        let description_param = match &category.description {
            Some(d) => libsql::Value::Text(d.clone()),
            None => libsql::Value::Null,
        };

        conn.execute(
            r"INSERT INTO categories (id, name, slug, description, created_at, updated_at)
              VALUES (?, ?, ?, ?, ?, ?)",
            vec![
                libsql::Value::Text(id.to_string()),
                libsql::Value::Text(category.name.clone()),
                libsql::Value::Text(slug.clone()),
                description_param,
                libsql::Value::Text(now.to_rfc3339()),
                libsql::Value::Text(now.to_rfc3339()),
            ],
        )
        .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::InternalError("Failed to create category".to_string()))
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<CategoryRecord>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT id, name, slug, description FROM categories WHERE id = ?",
                libsql::params![id.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            let name: String = row.get(1)?;
            let slug: String = row.get(2)?;
            let description: Option<String> = row.get(3).ok();

            Ok(Some(CategoryRecord {
                id,
                name,
                slug,
                description,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_slug(&self, slug: &str) -> RepoResult<Option<CategoryRecord>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT id, name, slug, description FROM categories WHERE slug = ?",
                libsql::params![slug],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            let name: String = row.get(1)?;
            let slug: String = row.get(2)?;
            let description: Option<String> = row.get(3).ok();

            Ok(Some(CategoryRecord {
                id,
                name,
                slug,
                description,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_slug_with_count(&self, slug: &str) -> RepoResult<Option<CategoryWithCount>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT c.id, c.name, c.slug, c.description, COUNT(a.id) as article_count
                  FROM categories c
                  LEFT JOIN articles a ON a.category_id = c.id AND a.draft = 0
                  WHERE c.slug = ?
                  GROUP BY c.id, c.name, c.slug, c.description",
                libsql::params![slug],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            let name: String = row.get(1)?;
            let slug: String = row.get(2)?;
            let description: Option<String> = row.get(3).ok();
            let article_count: i64 = row.get(4).unwrap_or(0);

            Ok(Some(CategoryWithCount {
                category: CategoryRecord {
                    id,
                    name,
                    slug,
                    description,
                },
                article_count,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_name(&self, name: &str) -> RepoResult<Option<CategoryRecord>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                "SELECT id, name, slug, description FROM categories WHERE name = ?",
                libsql::params![name],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            let name: String = row.get(1)?;
            let slug: String = row.get(2)?;
            let description: Option<String> = row.get(3).ok();

            Ok(Some(CategoryRecord {
                id,
                name,
                slug,
                description,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_with_counts(&self) -> RepoResult<Vec<CategoryWithCount>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let mut rows = conn
            .query(
                r"SELECT c.id, c.name, c.slug, c.description, COUNT(a.id) as article_count
                  FROM categories c
                  LEFT JOIN articles a ON a.category_id = c.id AND a.draft = 0
                  GROUP BY c.id, c.name, c.slug, c.description
                  ORDER BY c.name ASC",
                (),
            )
            .await?;

        let mut categories = Vec::new();
        while let Some(row) = rows.next().await? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RepositoryError::InternalError(format!("Invalid UUID: {}", e)))?;
            let name: String = row.get(1)?;
            let slug: String = row.get(2)?;
            let description: Option<String> = row.get(3).ok();
            let article_count: i64 = row.get(4).unwrap_or(0);

            categories.push(CategoryWithCount {
                category: CategoryRecord {
                    id,
                    name,
                    slug,
                    description,
                },
                article_count,
            });
        }

        Ok(categories)
    }

    async fn update(&self, slug: &str, data: &UpdateCategoryData) -> RepoResult<CategoryRecord> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        let existing = self.find_by_slug(slug).await?;
        if existing.is_none() {
            return Err(RepositoryError::NotFound(format!(
                "Category with slug '{}'",
                slug
            )));
        }

        let mut updates = Vec::new();
        let mut params: Vec<libsql::Value> = Vec::new();

        if let Some(name) = &data.name {
            updates.push("name = ?");
            params.push(libsql::Value::Text(name.clone()));
            // Also update the slug when name changes
            updates.push("slug = ?");
            params.push(libsql::Value::Text(slugify(name)));
        }
        if let Some(description) = &data.description {
            updates.push("description = ?");
            params.push(libsql::Value::Text(description.clone()));
        }

        if updates.is_empty() {
            return self.find_by_slug(slug).await?.ok_or_else(|| {
                RepositoryError::InternalError("Category disappeared during update".to_string())
            });
        }

        updates.push("updated_at = ?");
        params.push(libsql::Value::Text(Utc::now().to_rfc3339()));

        params.push(libsql::Value::Text(slug.to_string()));

        let update_query = format!(
            "UPDATE categories SET {} WHERE slug = ?",
            updates.join(", ")
        );

        conn.execute(&update_query, libsql::params_from_iter(params))
            .await?;

        // If name was updated, the slug changed too
        let new_slug = if let Some(name) = &data.name {
            slugify(name)
        } else {
            slug.to_string()
        };

        self.find_by_slug(&new_slug).await?.ok_or_else(|| {
            RepositoryError::InternalError("Failed to retrieve updated category".to_string())
        })
    }

    async fn delete(&self, slug: &str) -> RepoResult<()> {
        let conn = self
            .db
            .connect()
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // First, unset category_id on any articles using this category
        let category = self.find_by_slug(slug).await?;
        if let Some(cat) = &category {
            conn.execute(
                "UPDATE articles SET category_id = NULL WHERE category_id = ?",
                libsql::params![cat.id.to_string()],
            )
            .await?;
        }

        let result = conn
            .execute(
                "DELETE FROM categories WHERE slug = ?",
                libsql::params![slug],
            )
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Category with slug '{}'",
                slug
            )));
        }

        Ok(())
    }
}
