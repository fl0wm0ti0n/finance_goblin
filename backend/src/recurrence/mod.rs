pub mod amount;
pub mod cadence;
pub mod detect;
pub mod group;
pub mod normalize;

pub use detect::{
    compute_fingerprint, detect_recurrence_groups, detect_recurrence_inflow_groups,
    RecurrenceConfig, RecurrenceGroup,
};
