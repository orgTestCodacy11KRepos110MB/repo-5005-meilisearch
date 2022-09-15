use std::ops::ControlFlow;

use crate::{task::Kind, TaskId};

#[derive(Debug)]
pub enum BatchKind {
    DocumentClear {
        ids: Vec<TaskId>,
    },
    DocumentAddition {
        addition_ids: Vec<TaskId>,
    },
    DocumentUpdate {
        update_ids: Vec<TaskId>,
    },
    DocumentDeletion {
        deletion_ids: Vec<TaskId>,
    },
    ClearAndSettings {
        other: Vec<TaskId>,
        settings_ids: Vec<TaskId>,
    },
    SettingsAndDocumentAddition {
        settings_ids: Vec<TaskId>,
        addition_ids: Vec<TaskId>,
    },
    SettingsAndDocumentUpdate {
        settings_ids: Vec<TaskId>,
        update_ids: Vec<TaskId>,
    },
    Settings {
        settings_ids: Vec<TaskId>,
    },
    IndexDeletion {
        ids: Vec<TaskId>,
    },
    IndexCreation {
        id: TaskId,
    },
    IndexUpdate {
        id: TaskId,
    },
    IndexRename {
        id: TaskId,
    },
    IndexSwap {
        id: TaskId,
    },
}

impl BatchKind {
    /// return true if you must stop right there.
    pub fn new(task_id: TaskId, kind: Kind) -> (Self, bool) {
        match kind {
            Kind::IndexCreation => (BatchKind::IndexCreation { id: task_id }, true),
            Kind::IndexDeletion => (BatchKind::IndexDeletion { ids: vec![task_id] }, true),
            Kind::IndexUpdate => (BatchKind::IndexUpdate { id: task_id }, true),
            Kind::IndexRename => (BatchKind::IndexRename { id: task_id }, true),
            Kind::IndexSwap => (BatchKind::IndexSwap { id: task_id }, true),
            Kind::DocumentClear => (BatchKind::DocumentClear { ids: vec![task_id] }, false),
            Kind::DocumentAddition => (
                BatchKind::DocumentAddition {
                    addition_ids: vec![task_id],
                },
                false,
            ),
            Kind::DocumentUpdate => (
                BatchKind::DocumentUpdate {
                    update_ids: vec![task_id],
                },
                false,
            ),
            Kind::DocumentDeletion => (
                BatchKind::DocumentDeletion {
                    deletion_ids: vec![task_id],
                },
                false,
            ),
            Kind::Settings => (
                BatchKind::Settings {
                    settings_ids: vec![task_id],
                },
                false,
            ),

            Kind::DumpExport | Kind::Snapshot | Kind::CancelTask => unreachable!(),
        }
    }

    /// Return true if you must stop.
    fn accumulate(self, id: TaskId, kind: Kind) -> ControlFlow<Self, Self> {
        match (self, kind) {
            // We don't batch any of these operations
            (
                this,
                Kind::IndexCreation | Kind::IndexRename | Kind::IndexUpdate | Kind::IndexSwap,
            ) => ControlFlow::Break(this),
            // The index deletion can batch with everything but must stop after
            (
                BatchKind::DocumentClear { mut ids }
                | BatchKind::DocumentAddition {
                    addition_ids: mut ids,
                }
                | BatchKind::DocumentUpdate {
                    update_ids: mut ids,
                }
                | BatchKind::DocumentDeletion {
                    deletion_ids: mut ids,
                }
                | BatchKind::Settings {
                    settings_ids: mut ids,
                },
                Kind::IndexDeletion,
            ) => {
                ids.push(id);
                ControlFlow::Break(BatchKind::IndexDeletion { ids })
            }
            (
                BatchKind::ClearAndSettings {
                    settings_ids: mut ids,
                    mut other,
                }
                | BatchKind::SettingsAndDocumentAddition {
                    addition_ids: mut ids,
                    settings_ids: mut other,
                }
                | BatchKind::SettingsAndDocumentUpdate {
                    update_ids: mut ids,
                    settings_ids: mut other,
                },
                Kind::IndexDeletion,
            ) => {
                ids.push(id);
                ids.append(&mut other);
                ControlFlow::Break(BatchKind::IndexDeletion { ids })
            }

            (
                BatchKind::DocumentClear { mut ids },
                Kind::DocumentClear | Kind::DocumentDeletion,
            ) => {
                ids.push(id);
                ControlFlow::Continue(BatchKind::DocumentClear { ids })
            }
            (
                this @ BatchKind::DocumentClear { .. },
                Kind::DocumentAddition | Kind::DocumentUpdate | Kind::Settings,
            ) => ControlFlow::Break(this),
            (
                BatchKind::DocumentAddition {
                    addition_ids: mut ids,
                }
                | BatchKind::DocumentUpdate {
                    update_ids: mut ids,
                },
                Kind::DocumentClear,
            ) => {
                ids.push(id);
                ControlFlow::Continue(BatchKind::DocumentClear { ids })
            }

            // we can autobatch the same kind of document additions / updates
            (BatchKind::DocumentAddition { mut addition_ids }, Kind::DocumentAddition) => {
                addition_ids.push(id);
                ControlFlow::Continue(BatchKind::DocumentAddition { addition_ids })
            }
            (BatchKind::DocumentUpdate { mut update_ids }, Kind::DocumentUpdate) => {
                update_ids.push(id);
                ControlFlow::Continue(BatchKind::DocumentUpdate { update_ids })
            }
            // but we can't autobatch documents if it's not the same kind
            // this match branch MUST be AFTER the previous one
            (
                this @ BatchKind::DocumentAddition { .. } | this @ BatchKind::DocumentUpdate { .. },
                Kind::DocumentDeletion | Kind::DocumentAddition | Kind::DocumentUpdate,
            ) => ControlFlow::Break(this),
            (BatchKind::DocumentAddition { addition_ids }, Kind::Settings) => {
                ControlFlow::Continue(BatchKind::SettingsAndDocumentAddition {
                    settings_ids: vec![id],
                    addition_ids,
                })
            }
            (BatchKind::DocumentUpdate { update_ids }, Kind::Settings) => {
                ControlFlow::Continue(BatchKind::SettingsAndDocumentUpdate {
                    settings_ids: vec![id],
                    update_ids,
                })
            }

            (BatchKind::DocumentDeletion { mut deletion_ids }, Kind::DocumentClear) => {
                deletion_ids.push(id);
                ControlFlow::Continue(BatchKind::DocumentClear { ids: deletion_ids })
            }
            (
                this @ BatchKind::DocumentDeletion { .. },
                Kind::DocumentAddition | Kind::DocumentUpdate,
            ) => ControlFlow::Break(this),
            (BatchKind::DocumentDeletion { mut deletion_ids }, Kind::DocumentDeletion) => {
                deletion_ids.push(id);
                ControlFlow::Continue(BatchKind::DocumentDeletion { deletion_ids })
            }
            (this @ BatchKind::DocumentDeletion { .. }, Kind::Settings) => ControlFlow::Break(this),

            (BatchKind::Settings { settings_ids }, Kind::DocumentClear) => {
                ControlFlow::Continue(BatchKind::ClearAndSettings {
                    settings_ids: settings_ids.clone(),
                    other: vec![id],
                })
            }
            (
                this @ BatchKind::Settings { .. },
                Kind::DocumentAddition | Kind::DocumentUpdate | Kind::DocumentDeletion,
            ) => ControlFlow::Break(this),
            (BatchKind::Settings { mut settings_ids }, Kind::Settings) => {
                settings_ids.push(id);
                ControlFlow::Continue(BatchKind::Settings { settings_ids })
            }

            (
                BatchKind::ClearAndSettings {
                    mut other,
                    settings_ids,
                },
                Kind::DocumentClear,
            ) => {
                other.push(id);
                ControlFlow::Continue(BatchKind::ClearAndSettings {
                    other,
                    settings_ids,
                })
            }
            (
                this @ BatchKind::ClearAndSettings { .. },
                Kind::DocumentAddition | Kind::DocumentUpdate,
            ) => ControlFlow::Break(this),
            (
                BatchKind::ClearAndSettings {
                    mut other,
                    settings_ids,
                },
                Kind::DocumentDeletion,
            ) => {
                other.push(id);
                ControlFlow::Continue(BatchKind::ClearAndSettings {
                    other,
                    settings_ids,
                })
            }
            (
                BatchKind::ClearAndSettings {
                    mut settings_ids,
                    other,
                },
                Kind::Settings,
            ) => {
                settings_ids.push(id);
                ControlFlow::Continue(BatchKind::ClearAndSettings {
                    other,
                    settings_ids,
                })
            }
            (
                BatchKind::SettingsAndDocumentAddition {
                    settings_ids,
                    addition_ids: mut other,
                }
                | BatchKind::SettingsAndDocumentUpdate {
                    settings_ids,
                    update_ids: mut other,
                },
                Kind::DocumentClear,
            ) => {
                other.push(id);

                ControlFlow::Continue(BatchKind::ClearAndSettings {
                    settings_ids,
                    other,
                })
            }

            // we can batch the settings with a kind of document operation with the same kind of document operation
            (
                BatchKind::SettingsAndDocumentAddition {
                    mut addition_ids,
                    settings_ids,
                },
                Kind::DocumentAddition,
            ) => {
                addition_ids.push(id);
                ControlFlow::Continue(BatchKind::SettingsAndDocumentAddition {
                    addition_ids,
                    settings_ids,
                })
            }
            (
                BatchKind::SettingsAndDocumentUpdate {
                    mut update_ids,
                    settings_ids,
                },
                Kind::DocumentUpdate,
            ) => {
                update_ids.push(id);
                ControlFlow::Continue(BatchKind::SettingsAndDocumentUpdate {
                    update_ids,
                    settings_ids,
                })
            }
            // But we can't batch a settings and a doc op with another doc op
            // this MUST be AFTER the two previous branch
            (
                this @ BatchKind::SettingsAndDocumentAddition { .. }
                | this @ BatchKind::SettingsAndDocumentUpdate { .. },
                Kind::DocumentDeletion | Kind::DocumentAddition | Kind::DocumentUpdate,
            ) => ControlFlow::Break(this),
            (
                BatchKind::SettingsAndDocumentAddition {
                    mut settings_ids,
                    addition_ids,
                },
                Kind::Settings,
            ) => {
                settings_ids.push(id);
                ControlFlow::Continue(BatchKind::SettingsAndDocumentAddition {
                    settings_ids,
                    addition_ids,
                })
            }
            (
                BatchKind::SettingsAndDocumentUpdate {
                    mut settings_ids,
                    update_ids,
                },
                Kind::Settings,
            ) => {
                settings_ids.push(id);
                ControlFlow::Continue(BatchKind::SettingsAndDocumentUpdate {
                    settings_ids,
                    update_ids,
                })
            }
            (_, Kind::CancelTask | Kind::DumpExport | Kind::Snapshot) => unreachable!(),
            (
                BatchKind::IndexCreation { .. }
                | BatchKind::IndexDeletion { .. }
                | BatchKind::IndexUpdate { .. }
                | BatchKind::IndexRename { .. }
                | BatchKind::IndexSwap { .. },
                _,
            ) => {
                unreachable!()
            }
        }
    }
}

pub fn autobatch(enqueued: Vec<(TaskId, Kind)>) -> Option<BatchKind> {
    let mut enqueued = enqueued.into_iter();
    let (id, kind) = enqueued.next()?;
    let (mut acc, is_finished) = BatchKind::new(id, kind);
    if is_finished {
        return Some(acc);
    }

    for (id, kind) in enqueued {
        acc = match acc.accumulate(id, kind) {
            ControlFlow::Continue(acc) => acc,
            ControlFlow::Break(acc) => return Some(acc),
        };
    }

    Some(acc)
}

#[cfg(test)]
mod tests {
    use crate::assert_smol_debug_snapshot;

    use super::*;
    use Kind::*;

    fn input_from(input: impl IntoIterator<Item = Kind>) -> Vec<(TaskId, Kind)> {
        input
            .into_iter()
            .enumerate()
            .map(|(id, kind)| (id as TaskId, kind))
            .collect()
    }

    #[test]
    fn autobatch_simple_operation_together() {
        // we can autobatch one or multiple DocumentAddition together
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition])), @"Some(DocumentAddition { addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, DocumentAddition, DocumentAddition])), @"Some(DocumentAddition { addition_ids: [0, 1, 2] })");
        // we can autobatch one or multiple DocumentUpdate together
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate])), @"Some(DocumentUpdate { update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, DocumentUpdate, DocumentUpdate])), @"Some(DocumentUpdate { update_ids: [0, 1, 2] })");
        // we can autobatch one or multiple DocumentDeletion together
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentDeletion])), @"Some(DocumentDeletion { deletion_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentDeletion, DocumentDeletion, DocumentDeletion])), @"Some(DocumentDeletion { deletion_ids: [0, 1, 2] })");
        // we can autobatch one or multiple Settings together
        assert_smol_debug_snapshot!(autobatch(input_from([Settings])), @"Some(Settings { settings_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([Settings, Settings, Settings])), @"Some(Settings { settings_ids: [0, 1, 2] })");
    }

    #[test]
    fn simple_document_operation_dont_autobatch_with_other() {
        // addition, updates and deletion can't batch together
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, DocumentUpdate])), @"Some(DocumentAddition { addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, DocumentDeletion])), @"Some(DocumentAddition { addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, DocumentAddition])), @"Some(DocumentUpdate { update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, DocumentDeletion])), @"Some(DocumentUpdate { update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentDeletion, DocumentAddition])), @"Some(DocumentDeletion { deletion_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentDeletion, DocumentUpdate])), @"Some(DocumentDeletion { deletion_ids: [0] })");

        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, IndexCreation])), @"Some(DocumentAddition { addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, IndexCreation])), @"Some(DocumentUpdate { update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentDeletion, IndexCreation])), @"Some(DocumentDeletion { deletion_ids: [0] })");

        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, IndexUpdate])), @"Some(DocumentAddition { addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, IndexUpdate])), @"Some(DocumentUpdate { update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentDeletion, IndexUpdate])), @"Some(DocumentDeletion { deletion_ids: [0] })");

        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, IndexRename])), @"Some(DocumentAddition { addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, IndexRename])), @"Some(DocumentUpdate { update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentDeletion, IndexRename])), @"Some(DocumentDeletion { deletion_ids: [0] })");

        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, IndexSwap])), @"Some(DocumentAddition { addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, IndexSwap])), @"Some(DocumentUpdate { update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentDeletion, IndexSwap])), @"Some(DocumentDeletion { deletion_ids: [0] })");
    }

    #[test]
    fn document_addition_batch_with_settings() {
        // simple case
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, Settings])), @"Some(SettingsAndDocumentAddition { settings_ids: [1], addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, Settings])), @"Some(SettingsAndDocumentUpdate { settings_ids: [1], update_ids: [0] })");

        // multiple settings and doc addition
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, DocumentAddition, Settings, Settings])), @"Some(SettingsAndDocumentAddition { settings_ids: [2, 3], addition_ids: [0, 1] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, DocumentAddition, Settings, Settings])), @"Some(SettingsAndDocumentAddition { settings_ids: [2, 3], addition_ids: [0, 1] })");

        // addition and setting unordered
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, Settings, DocumentAddition, Settings])), @"Some(SettingsAndDocumentAddition { settings_ids: [1, 3], addition_ids: [0, 2] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, Settings, DocumentUpdate, Settings])), @"Some(SettingsAndDocumentUpdate { settings_ids: [1, 3], update_ids: [0, 2] })");

        // We ensure this kind of batch doesn't batch with forbidden operations
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, Settings, DocumentUpdate])), @"Some(SettingsAndDocumentAddition { settings_ids: [1], addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, Settings, DocumentAddition])), @"Some(SettingsAndDocumentUpdate { settings_ids: [1], update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, Settings, DocumentDeletion])), @"Some(SettingsAndDocumentAddition { settings_ids: [1], addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, Settings, DocumentDeletion])), @"Some(SettingsAndDocumentUpdate { settings_ids: [1], update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, Settings, IndexCreation])), @"Some(SettingsAndDocumentAddition { settings_ids: [1], addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, Settings, IndexCreation])), @"Some(SettingsAndDocumentUpdate { settings_ids: [1], update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, Settings, IndexUpdate])), @"Some(SettingsAndDocumentAddition { settings_ids: [1], addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, Settings, IndexUpdate])), @"Some(SettingsAndDocumentUpdate { settings_ids: [1], update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, Settings, IndexRename])), @"Some(SettingsAndDocumentAddition { settings_ids: [1], addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, Settings, IndexRename])), @"Some(SettingsAndDocumentUpdate { settings_ids: [1], update_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, Settings, IndexSwap])), @"Some(SettingsAndDocumentAddition { settings_ids: [1], addition_ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, Settings, IndexSwap])), @"Some(SettingsAndDocumentUpdate { settings_ids: [1], update_ids: [0] })");
    }

    #[test]
    fn clear_and_additions() {
        // these two doesn't need to batch
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentClear, DocumentAddition])), @"Some(DocumentClear { ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentClear, DocumentUpdate])), @"Some(DocumentClear { ids: [0] })");

        // Basic use case
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, DocumentAddition, DocumentClear])), @"Some(DocumentClear { ids: [0, 1, 2] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, DocumentUpdate, DocumentClear])), @"Some(DocumentClear { ids: [0, 1, 2] })");

        // This batch kind doesn't mix with other document addition
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, DocumentAddition, DocumentClear, DocumentAddition])), @"Some(DocumentClear { ids: [0, 1, 2] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, DocumentUpdate, DocumentClear, DocumentUpdate])), @"Some(DocumentClear { ids: [0, 1, 2] })");

        // But you can batch multiple clear together
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, DocumentAddition, DocumentClear, DocumentClear, DocumentClear])), @"Some(DocumentClear { ids: [0, 1, 2, 3, 4] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, DocumentUpdate, DocumentClear, DocumentClear, DocumentClear])), @"Some(DocumentClear { ids: [0, 1, 2, 3, 4] })");
    }

    #[test]
    fn clear_and_additions_and_settings() {
        // A clear don't need to autobatch the settings that happens AFTER there is no documents
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentClear, Settings])), @"Some(DocumentClear { ids: [0] })");

        assert_smol_debug_snapshot!(autobatch(input_from([Settings, DocumentClear, Settings])), @"Some(ClearAndSettings { other: [1], settings_ids: [0, 2] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, Settings, DocumentClear])), @"Some(ClearAndSettings { other: [0, 2], settings_ids: [1] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, Settings, DocumentClear])), @"Some(ClearAndSettings { other: [0, 2], settings_ids: [1] })");
    }

    #[test]
    fn anything_and_index_deletion() {
        // The indexdeletion doesn't batch with anything that happens AFTER
        assert_smol_debug_snapshot!(autobatch(input_from([IndexDeletion, DocumentAddition])), @"Some(IndexDeletion { ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([IndexDeletion, DocumentUpdate])), @"Some(IndexDeletion { ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([IndexDeletion, DocumentDeletion])), @"Some(IndexDeletion { ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([IndexDeletion, DocumentClear])), @"Some(IndexDeletion { ids: [0] })");
        assert_smol_debug_snapshot!(autobatch(input_from([IndexDeletion, Settings])), @"Some(IndexDeletion { ids: [0] })");

        // The index deletion can accept almost any type of BatchKind and transform it to an IndexDeletion
        // First, the basic cases
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, IndexDeletion])), @"Some(IndexDeletion { ids: [0, 1] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, IndexDeletion])), @"Some(IndexDeletion { ids: [0, 1] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentDeletion, IndexDeletion])), @"Some(IndexDeletion { ids: [0, 1] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentClear, IndexDeletion])), @"Some(IndexDeletion { ids: [0, 1] })");
        assert_smol_debug_snapshot!(autobatch(input_from([Settings, IndexDeletion])), @"Some(IndexDeletion { ids: [0, 1] })");

        // Then the mixed cases
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, Settings, IndexDeletion])), @"Some(IndexDeletion { ids: [0, 2, 1] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, Settings, IndexDeletion])), @"Some(IndexDeletion { ids: [0, 2, 1] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentAddition, Settings, DocumentClear, IndexDeletion])), @"Some(IndexDeletion { ids: [1, 3, 0, 2] })");
        assert_smol_debug_snapshot!(autobatch(input_from([DocumentUpdate, Settings, DocumentClear, IndexDeletion])), @"Some(IndexDeletion { ids: [1, 3, 0, 2] })");
    }
}