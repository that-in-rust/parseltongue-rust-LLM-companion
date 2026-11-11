#![allow(clippy::unwrap_used)]

use parseltongue_core_row_types::{RunCancelFlagValue, StoreBackendFailureError};
use parseltongue_stage_worker_scheduler::run_bounded_stage_workers;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn test_pure_worker_results_ordered() {
    use parseltongue_stage_worker_scheduler::map_bounded_stage_workers;
    let keys = vec!["one".into(), "two".into(), "three".into()];
    let peak = AtomicUsize::new(0);
    let active = AtomicUsize::new(0);
    let output = map_bounded_stage_workers(&keys,2,&RunCancelFlagValue::default(),|key|{
        let count = active.fetch_add(1,Ordering::SeqCst)+1;
        peak.fetch_max(count,Ordering::SeqCst);
        std::thread::yield_now();
        active.fetch_sub(1,Ordering::SeqCst);
        Ok(key.len())
    }).unwrap();
    assert_eq!(output,vec![3,3,5]);
    assert!(peak.load(Ordering::SeqCst)<=2);
    let output: Vec<usize> = map_bounded_stage_workers(&[],2,&RunCancelFlagValue::default(),|_|Ok(0)).unwrap();
    assert!(output.is_empty());
}

#[test]
fn test_pure_worker_errors_propagate() {
    use parseltongue_stage_worker_scheduler::map_bounded_stage_workers;
    let keys = vec!["one".into(),"two".into()];
    assert!(map_bounded_stage_workers(&keys,0,&RunCancelFlagValue::default(),|_|Ok(0)).is_err());
    assert!(map_bounded_stage_workers::<()>(&keys,2,&RunCancelFlagValue::default(),|_|Err(StoreBackendFailureError{error_context_detail_text:"failure".into()})).is_err());
    let cancelled = RunCancelFlagValue::default();
    cancelled.request_stage_work_cancellation();
    assert!(map_bounded_stage_workers(&keys,2,&cancelled,|_|Ok(0)).is_err());
    assert!(map_bounded_stage_workers::<()>(&keys,2,&RunCancelFlagValue::default(),|key|{
        if key=="one" {panic!("worker failure");}
        Err(StoreBackendFailureError{error_context_detail_text:"other failure".into()})
    }).is_err());
}

#[test]
fn test_in_flight_units_bounded() {
    let active = AtomicUsize::new(0);
    let peak = AtomicUsize::new(0);
    let finished = AtomicUsize::new(0);
    let jobs: Vec<_> = (0..24).map(|n| n.to_string()).collect();
    run_bounded_stage_workers(&jobs, 3, &RunCancelFlagValue::default(), |_| {
        let count = active.fetch_add(1, Ordering::SeqCst) + 1;
        peak.fetch_max(count, Ordering::SeqCst);
        std::thread::yield_now();
        active.fetch_sub(1, Ordering::SeqCst);
        finished.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .unwrap();
    assert!(peak.load(Ordering::SeqCst) <= 3);
    assert_eq!(finished.load(Ordering::SeqCst), 24);
    assert!(run_bounded_stage_workers(&[], 0, &RunCancelFlagValue::default(), |_| Ok(())).is_err());
    assert!(run_bounded_stage_workers(&[], 1, &RunCancelFlagValue::default(), |_| Ok(())).is_ok());
    assert!(
        run_bounded_stage_workers(&jobs, 1, &RunCancelFlagValue::default(), |_| Err(
            StoreBackendFailureError {
                error_context_detail_text: "injected failure".into(),
            }
        ))
        .is_err()
    );
}

#[test]
fn test_cancel_then_rerun_identical() {
    let cancelled = RunCancelFlagValue::default();
    cancelled.request_stage_work_cancellation();
    let jobs = vec!["one".into(), "two".into()];
    assert!(
        run_bounded_stage_workers(&jobs, 2, &cancelled, |_| panic!("cancelled task ran")).is_err()
    );
    let finished = AtomicUsize::new(0);
    run_bounded_stage_workers(&jobs, 2, &RunCancelFlagValue::default(), |_| {
        finished.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .unwrap();
    assert_eq!(finished.load(Ordering::SeqCst), 2);
}
