// src/lib/models/role.rs

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// User roles in the system with hierarchical permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Full system access, user management, role assignment
    Admin,
    /// Can write and manage own articles, has subscriber permissions
    Author,
    /// Can read content, manage own account
    Subscriber,
}

impl Role {
    /// Check if this role has at least the permissions of the given role
    pub fn has_permission(&self, required_role: Role) -> bool {
        match (self, required_role) {
            // Admin has all permissions
            (Role::Admin, _) => true,
            // Author has author and subscriber permissions
            (Role::Author, Role::Author) | (Role::Author, Role::Subscriber) => true,
            // Subscriber only has subscriber permissions
            (Role::Subscriber, Role::Subscriber) => true,
            // All other cases are denied
            _ => false,
        }
    }

    /// Check if user is an admin
    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Check if user is an author (or higher)
    pub fn is_author(&self) -> bool {
        matches!(self, Role::Admin | Role::Author)
    }

    /// Check if user is at least a subscriber
    pub fn is_subscriber(&self) -> bool {
        matches!(self, Role::Admin | Role::Author | Role::Subscriber)
    }

    /// Get all available roles
    pub fn all() -> Vec<Role> {
        vec![Role::Subscriber, Role::Author, Role::Admin]
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Author => write!(f, "author"),
            Role::Subscriber => write!(f, "subscriber"),
        }
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "author" => Ok(Role::Author),
            "subscriber" => Ok(Role::Subscriber),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

impl Default for Role {
    fn default() -> Self {
        Role::Subscriber
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_permissions() {
        // Admin has all permissions
        assert!(Role::Admin.has_permission(Role::Admin));
        assert!(Role::Admin.has_permission(Role::Author));
        assert!(Role::Admin.has_permission(Role::Subscriber));

        // Author has author and subscriber permissions
        assert!(!Role::Author.has_permission(Role::Admin));
        assert!(Role::Author.has_permission(Role::Author));
        assert!(Role::Author.has_permission(Role::Subscriber));

        // Subscriber only has subscriber permissions
        assert!(!Role::Subscriber.has_permission(Role::Admin));
        assert!(!Role::Subscriber.has_permission(Role::Author));
        assert!(Role::Subscriber.has_permission(Role::Subscriber));
    }

    #[test]
    fn test_role_checks() {
        assert!(Role::Admin.is_admin());
        assert!(!Role::Author.is_admin());
        assert!(!Role::Subscriber.is_admin());

        assert!(Role::Admin.is_author());
        assert!(Role::Author.is_author());
        assert!(!Role::Subscriber.is_author());

        assert!(Role::Admin.is_subscriber());
        assert!(Role::Author.is_subscriber());
        assert!(Role::Subscriber.is_subscriber());
    }

    #[test]
    fn test_role_display() {
        assert_eq!(Role::Admin.to_string(), "admin");
        assert_eq!(Role::Author.to_string(), "author");
        assert_eq!(Role::Subscriber.to_string(), "subscriber");
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("admin").unwrap(), Role::Admin);
        assert_eq!(Role::from_str("ADMIN").unwrap(), Role::Admin);
        assert_eq!(Role::from_str("author").unwrap(), Role::Author);
        assert_eq!(Role::from_str("Author").unwrap(), Role::Author);
        assert_eq!(Role::from_str("subscriber").unwrap(), Role::Subscriber);
        assert_eq!(Role::from_str("SUBSCRIBER").unwrap(), Role::Subscriber);
        assert!(Role::from_str("invalid").is_err());
    }

    #[test]
    fn test_role_default() {
        assert_eq!(Role::default(), Role::Subscriber);
    }
}
