use cea_sys::*;
use std::ptr;

fn assert_ok(err: cea_err, what: &str) {
    // If you later want to use the real success constant, grep it from bindings.rs,
    // but "0 means success" is extremely common and works well for smoke tests.
    assert_eq!(err as i32, 0, "{what} failed: err={}", err as i32);
}

#[test]
fn can_create_and_destroy_shock_solution() {
    unsafe {
        let mut soln: cea_shock_solution = ptr::null_mut();
        assert_ok(cea_shock_solution_create(&mut soln, 1), "cea_shock_solution_create");
        assert!(!soln.is_null(), "shock solution handle should not be null after create");
        assert_ok(cea_shock_solution_destroy(&mut soln), "cea_shock_solution_destroy");
        assert!(soln.is_null(), "shock solution handle should be null after destroy");
    }
}

#[test]
fn can_create_and_destroy_detonation_solution() {
    unsafe {
        let mut soln: cea_detonation_solution = ptr::null_mut();
        assert_ok(
            cea_detonation_solution_create(&mut soln),
            "cea_detonation_solution_create",
        );
        assert!(!soln.is_null(), "detonation solution handle should not be null after create");
        assert_ok(
            cea_detonation_solution_destroy(&mut soln),
            "cea_detonation_solution_destroy",
        );
        assert!(soln.is_null(), "detonation solution handle should be null after destroy");
    }
}
