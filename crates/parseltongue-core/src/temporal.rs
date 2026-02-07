//! Temporal versioning system for Parseltongue.
//!
//! Implements the core temporal versioning logic with state transitions,
//! consistency validation, and conflict resolution.

use crate::entities::*;
use crate::error::{ParseltongError, Result};
use crate::interfaces::*;
use std::collections::HashMap;
use std::fmt;

/// Temporal versioning manager
///
/// Handles state transitions, consistency validation, and conflict resolution
/// for the temporal versioning system.
#[derive(Debug)]
pub struct TemporalVersioningManager {
    /// Current state of all entities
    entities: HashMap<String, CodeEntity>,
    /// Pending changes not yet applied
    #[allow(dead_code)]
    pending_changes: Vec<TemporalChange>,
    /// Validation rules
    validation_rules: Vec<Box<dyn TemporalValidationRule>>,
}

impl TemporalVersioningManager {
    /// Create new temporal versioning manager
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            pending_changes: Vec::new(),
            validation_rules: vec![
                Box::new(NoCircularDependenciesRule::new()),
                Box::new(ConsistentStateRule::new()),
                Box::new(ValidTransitionsRule::new()),
            ],
        }
    }

    /// Add an entity to the temporal state
    pub fn add_entity(&mut self, entity: CodeEntity) -> Result<()> {
        // Validate entity
        entity.validate()?;

        // Check for conflicts with existing entity
        if let Some(existing) = self.entities.get(&entity.isgl1_key) {
            self.validate_entity_compatibility(existing, &entity)?;
        }

        self.entities.insert(entity.isgl1_key.clone(), entity);
        Ok(())
    }

    /// Apply temporal changes to entities
    pub fn apply_changes(&mut self, changes: Vec<TemporalChange>) -> Result<Vec<String>> {
        let mut affected_entities = Vec::new();

        for change in changes {
            let isgl1_key = &change.isgl1_key;

            // Validate change
            self.validate_temporal_change(&change)?;

            // Apply change to entity
            if let Some(entity) = self.entities.get_mut(isgl1_key) {
                entity.apply_temporal_change(change.action.clone(), change.future_code.clone())?;
                affected_entities.push(isgl1_key.clone());
            } else {
                // Entity doesn't exist, create new one
                let mut entity = self.create_entity_for_change(&change)?;
                entity.apply_temporal_change(change.action.clone(), change.future_code.clone())?;
                self.entities.insert(isgl1_key.clone(), entity);
                affected_entities.push(isgl1_key.clone());
            }
        }

        // Run validation rules
        self.run_validation_rules()?;

        Ok(affected_entities)
    }

    /// Reset temporal state (Tool 6 operation)
    pub fn reset_temporal_state(&mut self) -> Result<usize> {
        let mut reset_count = 0;

        for entity in self.entities.values_mut() {
            if entity.is_modified() {
                // Apply temporal state changes
                match &entity.temporal_state.future_action {
                    Some(TemporalAction::Create) => {
                        // New entity becomes current
                        entity.temporal_state.current_ind = true;
                        entity.current_code = entity.future_code.clone();
                    }
                    Some(TemporalAction::Edit) => {
                        // Apply edit
                        entity.current_code = entity.future_code.clone();
                    }
                    Some(TemporalAction::Delete) => {
                        // Mark for deletion (will be removed by caller)
                        entity.temporal_state.current_ind = false;
                    }
                    None => {}
                }

                // Reset temporal indicators
                entity.temporal_state.future_ind = entity.temporal_state.current_ind;
                entity.temporal_state.future_action = None;
                entity.future_code = None;

                reset_count += 1;
            }
        }

        // Remove deleted entities
        self.entities.retain(|_, entity| entity.temporal_state.current_ind);

        Ok(reset_count)
    }

    /// Get entities with pending changes
    pub fn get_changed_entities(&self) -> Vec<&CodeEntity> {
        self.entities
            .values()
            .filter(|entity| entity.is_modified())
            .collect()
    }

    /// Get entity by ISGL1 key
    pub fn get_entity(&self, isgl1_key: &str) -> Option<&CodeEntity> {
        self.entities.get(isgl1_key)
    }

    /// Get all entities
    pub fn get_all_entities(&self) -> Vec<&CodeEntity> {
        self.entities.values().collect()
    }

    /// Validate temporal state consistency
    pub fn validate_state(&self) -> Result<()> {
        self.run_validation_rules()
    }

    /// Get entities that depend on a given entity
    pub fn get_dependents(&self, isgl1_key: &str) -> Vec<String> {
        self.entities
            .values()
            .filter(|entity| {
                // Check if entity depends on the given entity
                // This is a simplified implementation
                entity.interface_signature.file_path
                    .to_string_lossy()
                    .contains(isgl1_key)
            })
            .map(|entity| entity.isgl1_key.clone())
            .collect()
    }

    // Private helper methods

    fn validate_entity_compatibility(&self, existing: &CodeEntity, new: &CodeEntity) -> Result<()> {
        // Check if both entities have conflicting temporal states
        if existing.is_modified() && new.is_modified() {
            return Err(ParseltongError::TemporalError {
                details: format!(
                    "Concurrent modifications detected for entity {}",
                    existing.isgl1_key
                ),
            });
        }

        Ok(())
    }

    fn validate_temporal_change(&self, change: &TemporalChange) -> Result<()> {
        // Validate temporal action compatibility
        let entity = self.entities.get(&change.isgl1_key);

        match (&entity, &change.action) {
            (None, TemporalAction::Edit | TemporalAction::Delete) => {
                return Err(ParseltongError::TemporalError {
                    details: format!(
                        "Cannot {} non-existent entity {}",
                        match change.action {
                            TemporalAction::Edit => "edit",
                            TemporalAction::Delete => "delete",
                            _ => unreachable!(),
                        },
                        change.isgl1_key
                    ),
                });
            }
            (Some(entity), TemporalAction::Create) => {
                if entity.temporal_state.current_ind {
                    return Err(ParseltongError::TemporalError {
                        details: format!(
                            "Cannot create entity {} that already exists",
                            change.isgl1_key
                        ),
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn create_entity_for_change(&self, change: &TemporalChange) -> Result<CodeEntity> {
        let mut entity = CodeEntity::new(
            change.isgl1_key.clone(),
            InterfaceSignature {
                entity_type: EntityType::Function, // Default
                name: "unknown".to_string(),
                visibility: Visibility::Private,
                file_path: std::path::PathBuf::new(),
                line_range: LineRange::new(1, 1)?,
                module_path: vec![],
                documentation: None,
                language_specific: LanguageSpecificSignature::Rust(RustSignature {
                    generics: vec![],
                    lifetimes: vec![],
                    where_clauses: vec![],
                    attributes: vec![],
                    trait_impl: None,
                }),
            },
            // v0.9.0: Default to CodeImplementation for temporal changes
            crate::entities::EntityClass::CodeImplementation,
        )?;

        // Set initial temporal state based on action
        entity.temporal_state = match change.action {
            TemporalAction::Create => TemporalState::create(),
            TemporalAction::Edit => TemporalState::edit(),
            TemporalAction::Delete => TemporalState::delete(),
        };

        Ok(entity)
    }

    fn run_validation_rules(&self) -> Result<()> {
        for rule in &self.validation_rules {
            rule.validate(&self.entities)?;
        }
        Ok(())
    }
}

impl Default for TemporalVersioningManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for temporal validation rules
pub trait TemporalValidationRule: Send + Sync + fmt::Debug {
    /// Validate the current state
    fn validate(&self, entities: &HashMap<String, CodeEntity>) -> Result<()>;
}

/// Rule to prevent circular dependencies
#[derive(Debug)]
pub struct NoCircularDependenciesRule {
    _private: (),
}

impl Default for NoCircularDependenciesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl NoCircularDependenciesRule {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl TemporalValidationRule for NoCircularDependenciesRule {
    fn validate(&self, entities: &HashMap<String, CodeEntity>) -> Result<()> {
        // Simplified circular dependency detection
        // In a real implementation, this would build a dependency graph
        // and check for cycles

        for entity in entities.values() {
            if entity.is_modified() {
                // Check if entity depends on itself (simplified)
                if entity.isgl1_key.contains(&entity.interface_signature.name) {
                    return Err(ParseltongError::TemporalError {
                        details: format!(
                            "Potential circular dependency detected for entity {}",
                            entity.isgl1_key
                        ),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Rule to ensure consistent state
#[derive(Debug)]
pub struct ConsistentStateRule {
    _private: (),
}

impl Default for ConsistentStateRule {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsistentStateRule {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl TemporalValidationRule for ConsistentStateRule {
    fn validate(&self, entities: &HashMap<String, CodeEntity>) -> Result<()> {
        for (key, entity) in entities {
            // Validate temporal state consistency
            entity.validate()?;

            // Ensure code consistency
            if entity.temporal_state.current_ind && entity.current_code.is_none() {
                return Err(ParseltongError::TemporalError {
                    details: format!(
                        "Entity {} has current_ind=true but no current_code",
                        key
                    ),
                });
            }

            if entity.temporal_state.future_ind && entity.future_code.is_none() {
                return Err(ParseltongError::TemporalError {
                    details: format!(
                        "Entity {} has future_ind=true but no future_code",
                        key
                    ),
                });
            }
        }

        Ok(())
    }
}

/// Rule to ensure valid state transitions
#[derive(Debug)]
pub struct ValidTransitionsRule {
    _private: (),
}

impl Default for ValidTransitionsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidTransitionsRule {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl TemporalValidationRule for ValidTransitionsRule {
    fn validate(&self, entities: &HashMap<String, CodeEntity>) -> Result<()> {
        for (key, entity) in entities {
            if let Some(ref action) = entity.temporal_state.future_action {
                // Validate action is compatible with temporal indicators
                action.validate_with_indicators(
                    entity.temporal_state.current_ind,
                    entity.temporal_state.future_ind,
                ).map_err(|e| ParseltongError::TemporalError {
                    details: format!(
                        "Invalid transition for entity {}: {}",
                        key,
                        e
                    ),
                })?;
            }
        }

        Ok(())
    }
}

/// Temporal state transition builder
///
/// Provides a fluent interface for building temporal state transitions
#[derive(Debug)]
pub struct TemporalTransitionBuilder {
    isgl1_key: String,
    action: Option<TemporalAction>,
    future_code: Option<String>,
    updated_signature: Option<InterfaceSignature>,
}

impl TemporalTransitionBuilder {
    /// Create new transition builder
    pub fn new(isgl1_key: String) -> Self {
        Self {
            isgl1_key,
            action: None,
            future_code: None,
            updated_signature: None,
        }
    }

    /// Set the action to perform
    pub fn action(mut self, action: TemporalAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Set the future code
    pub fn future_code(mut self, code: String) -> Self {
        self.future_code = Some(code);
        self
    }

    /// Set the updated signature
    pub fn updated_signature(mut self, signature: InterfaceSignature) -> Self {
        self.updated_signature = Some(signature);
        self
    }

    /// Build the temporal change
    pub fn build(self) -> Result<TemporalChange> {
        let action = self.action.ok_or_else(|| ParseltongError::TemporalError {
            details: "Temporal action is required".to_string(),
        })?;

        Ok(TemporalChange {
            isgl1_key: self.isgl1_key,
            action,
            future_code: self.future_code,
            updated_signature: self.updated_signature,
        })
    }
}

/// Conflict resolution strategies
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolutionStrategy {
    /// Fail fast on any conflict
    FailFast,
    /// Use latest change
    UseLatest,
    /// Use earliest change
    UseEarliest,
    /// Merge changes if possible
    AttemptMerge,
}

/// Conflict detector and resolver
#[derive(Debug)]
pub struct ConflictResolver {
    strategy: ConflictResolutionStrategy,
}

impl ConflictResolver {
    /// Create new conflict resolver
    pub fn new(strategy: ConflictResolutionStrategy) -> Self {
        Self { strategy }
    }

    /// Detect conflicts between changes
    pub fn detect_conflicts(&self, changes: &[TemporalChange]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        // Check for multiple changes to same entity
        let mut entity_changes: HashMap<String, Vec<&TemporalChange>> = HashMap::new();
        for change in changes {
            entity_changes
                .entry(change.isgl1_key.clone())
                .or_default()
                .push(change);
        }

        for (entity, entity_changes) in entity_changes {
            if entity_changes.len() > 1 {
                conflicts.push(Conflict::MultipleChanges {
                    entity,
                    changes: entity_changes.iter().map(|c| (*c).clone()).collect(),
                });
            }
        }

        conflicts
    }

    /// Resolve conflicts using configured strategy
    pub fn resolve_conflicts(&self, changes: Vec<TemporalChange>) -> Result<Vec<TemporalChange>> {
        let conflicts = self.detect_conflicts(&changes);

        if conflicts.is_empty() {
            return Ok(changes);
        }

        match self.strategy {
            ConflictResolutionStrategy::FailFast => {
                Err(ParseltongError::TemporalError {
                    details: format!(
                        "Conflicts detected: {:?}",
                        conflicts
                    ),
                })
            }
            ConflictResolutionStrategy::UseLatest => {
                self.resolve_with_latest(changes, conflicts)
            }
            ConflictResolutionStrategy::UseEarliest => {
                self.resolve_with_earliest(changes, conflicts)
            }
            ConflictResolutionStrategy::AttemptMerge => {
                self.attempt_merge(changes, conflicts)
            }
        }
    }

    fn resolve_with_latest(&self, changes: Vec<TemporalChange>, conflicts: Vec<Conflict>) -> Result<Vec<TemporalChange>> {
        let mut resolved = changes;
        let mut to_remove = Vec::new();

        for conflict in conflicts {
            if let Conflict::MultipleChanges { changes: conflicting_changes, .. } = conflict {
                // Keep only the last change - remove all but the last
                for change in conflicting_changes.iter().take(conflicting_changes.len().saturating_sub(1)) {
                    to_remove.push(change.isgl1_key.clone());
                }
            }
        }

        resolved.retain(|change| !to_remove.contains(&change.isgl1_key));
        Ok(resolved)
    }

    fn resolve_with_earliest(&self, changes: Vec<TemporalChange>, conflicts: Vec<Conflict>) -> Result<Vec<TemporalChange>> {
        let mut resolved = changes;
        let mut to_remove = Vec::new();

        for conflict in conflicts {
            if let Conflict::MultipleChanges { changes: conflicting_changes, .. } = conflict {
                // Keep only the first change - remove all but the first
                for change in conflicting_changes.iter().skip(1) {
                    to_remove.push(change.isgl1_key.clone());
                }
            }
        }

        resolved.retain(|change| !to_remove.contains(&change.isgl1_key));
        Ok(resolved)
    }

    fn attempt_merge(&self, _changes: Vec<TemporalChange>, _conflicts: Vec<Conflict>) -> Result<Vec<TemporalChange>> {
        // Simplified merge implementation
        // In a real implementation, this would be more sophisticated
        Err(ParseltongError::TemporalError {
            details: "Merge conflict resolution not yet implemented".to_string(),
        })
    }
}

/// Conflict types
#[derive(Debug, Clone)]
pub enum Conflict {
    /// Multiple changes to the same entity
    MultipleChanges {
        entity: String,
        changes: Vec<TemporalChange>,
    },
    /// Dependency conflict
    DependencyConflict {
        entities: Vec<String>,
        description: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporal_state_validation() {
        let manager = TemporalVersioningManager::new();
        assert!(manager.validate_state().is_ok());
    }

    #[test]
    fn entity_creation_and_modification() {
        let mut manager = TemporalVersioningManager::new();

        // Create entity
        let mut entity = CodeEntity::new(
            "test.rs-compute_result".to_string(),
            InterfaceSignature {
                entity_type: EntityType::Function,
                name: "calculate_value".to_string(),
                visibility: Visibility::Public,
                file_path: std::path::PathBuf::from("test.rs"),
                line_range: LineRange::new(1, 5).unwrap(),
                module_path: vec![],
                documentation: None,
                language_specific: LanguageSpecificSignature::Rust(RustSignature {
                    generics: vec![],
                    lifetimes: vec![],
                    where_clauses: vec![],
                    attributes: vec![],
                    trait_impl: None,
                }),
            },
            // v0.9.0: Default to CodeImplementation for tests
            crate::entities::EntityClass::CodeImplementation,
        ).unwrap();

        // Set current_code and future_code to satisfy validation requirements
        entity.current_code = Some("fn test() {}".to_string());
        entity.future_code = Some("fn test() {}".to_string());

        // Set to unchanged state since both codes are the same
        entity.temporal_state = TemporalState::unchanged();

        manager.add_entity(entity).unwrap();

        // Apply edit change
        let changes = vec![TemporalChange {
            isgl1_key: "test.rs-compute_result".to_string(),
            action: TemporalAction::Edit,
            future_code: Some("fn test() {}".to_string()),
            updated_signature: None,
        }];

        let affected = manager.apply_changes(changes).unwrap();
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0], "test.rs-compute_result");

        let changed_entities = manager.get_changed_entities();
        assert_eq!(changed_entities.len(), 1);
    }

    #[test]
    fn temporal_transition_builder() {
        let transition = TemporalTransitionBuilder::new("test.rs-test".to_string())
            .action(TemporalAction::Create)
            .future_code("fn test() {}".to_string())
            .build()
            .unwrap();

        assert_eq!(transition.isgl1_key, "test.rs-test");
        assert_eq!(transition.action, TemporalAction::Create);
        assert_eq!(transition.future_code, Some("fn test() {}".to_string()));
    }

    #[test]
    fn conflict_detection() {
        let resolver = ConflictResolver::new(ConflictResolutionStrategy::FailFast);

        let changes = vec![
            TemporalChange {
                isgl1_key: "test.rs-function".to_string(),
                action: TemporalAction::Edit,
                future_code: Some("fn test() {}".to_string()),
                updated_signature: None,
            },
            TemporalChange {
                isgl1_key: "test.rs-function".to_string(),
                action: TemporalAction::Delete,
                future_code: None,
                updated_signature: None,
            },
        ];

        let conflicts = resolver.detect_conflicts(&changes);
        assert_eq!(conflicts.len(), 1);

        let result = resolver.resolve_conflicts(changes);
        assert!(result.is_err());
    }

    #[test]
    fn validation_rules() {
        let mut manager = TemporalVersioningManager::new();

        // Test with invalid entity (missing code when current_ind=true)
        let mut invalid_entity = CodeEntity::new(
            "invalid.rs-invalid".to_string(),
            InterfaceSignature {
                entity_type: EntityType::Function,
                name: "invalid".to_string(),
                visibility: Visibility::Public,
                file_path: std::path::PathBuf::from("invalid.rs"),
                line_range: LineRange::new(1, 1).unwrap(),
                module_path: vec![],
                documentation: None,
                language_specific: LanguageSpecificSignature::Rust(RustSignature {
                    generics: vec![],
                    lifetimes: vec![],
                    where_clauses: vec![],
                    attributes: vec![],
                    trait_impl: None,
                }),
            },
            // v0.9.0: Default to CodeImplementation for tests
            crate::entities::EntityClass::CodeImplementation,
        ).unwrap();

        invalid_entity.temporal_state.current_ind = true;
        // current_code is None, which should cause validation to fail

        let result = manager.add_entity(invalid_entity);
        assert!(result.is_err());
    }
}