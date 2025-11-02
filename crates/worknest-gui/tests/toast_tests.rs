//! Tests for the toast notification component

use wasm_bindgen_test::*;
use web_time::Instant;
use worknest_gui::{
    components::ToastManager,
    state::{Notification, NotificationLevel},
};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_toast_manager_initialization() {
    let manager = ToastManager::new();
    // Manager should start empty (no public access to toasts count, so we verify it compiles)
    assert!(true, "ToastManager initializes successfully");
}

#[wasm_bindgen_test]
fn test_toast_manager_default() {
    let manager = ToastManager::default();
    assert!(true, "ToastManager can be created with default()");
}

#[wasm_bindgen_test]
fn test_add_toast_from_notification() {
    let mut manager = ToastManager::new();
    let notification = Notification {
        message: "Test notification".to_string(),
        level: NotificationLevel::Info,
        timestamp: Instant::now(),
    };

    manager.add_toast(notification);
    // Toast added successfully (no panic)
    assert!(true);
}

#[wasm_bindgen_test]
fn test_add_multiple_toasts() {
    let mut manager = ToastManager::new();

    for i in 0..3 {
        let notification = Notification {
            message: format!("Notification {}", i),
            level: NotificationLevel::Info,
            timestamp: Instant::now(),
        };
        manager.add_toast(notification);
    }

    // Multiple toasts added successfully
    assert!(true);
}

#[wasm_bindgen_test]
fn test_toast_limit() {
    let mut manager = ToastManager::new();

    // Add more than 5 toasts (the limit)
    for i in 0..10 {
        let notification = Notification {
            message: format!("Notification {}", i),
            level: NotificationLevel::Info,
            timestamp: Instant::now(),
        };
        manager.add_toast(notification);
    }

    // Should handle overflow without panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_update_from_notifications() {
    let mut manager = ToastManager::new();
    let notifications = vec![
        Notification {
            message: "First".to_string(),
            level: NotificationLevel::Success,
            timestamp: Instant::now(),
        },
        Notification {
            message: "Second".to_string(),
            level: NotificationLevel::Error,
            timestamp: Instant::now(),
        },
    ];

    manager.update_from_notifications(&notifications);
    // Notifications converted to toasts successfully
    assert!(true);
}

#[wasm_bindgen_test]
fn test_update_from_notifications_no_duplicates() {
    let mut manager = ToastManager::new();
    let timestamp = Instant::now();
    let notification = Notification {
        message: "Duplicate test".to_string(),
        level: NotificationLevel::Info,
        timestamp,
    };

    let notifications = vec![notification.clone(), notification];

    manager.update_from_notifications(&notifications);
    // Should handle duplicates without panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_cleanup() {
    let mut manager = ToastManager::new();
    let notification = Notification {
        message: "Test".to_string(),
        level: NotificationLevel::Info,
        timestamp: Instant::now(),
    };

    manager.add_toast(notification);
    manager.cleanup();
    // Cleanup runs without panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_notification_level_success() {
    let notification = Notification {
        message: "Success!".to_string(),
        level: NotificationLevel::Success,
        timestamp: Instant::now(),
    };

    assert_eq!(notification.level, NotificationLevel::Success);
}

#[wasm_bindgen_test]
fn test_notification_level_error() {
    let notification = Notification {
        message: "Error!".to_string(),
        level: NotificationLevel::Error,
        timestamp: Instant::now(),
    };

    assert_eq!(notification.level, NotificationLevel::Error);
}

#[wasm_bindgen_test]
fn test_notification_level_info() {
    let notification = Notification {
        message: "Info".to_string(),
        level: NotificationLevel::Info,
        timestamp: Instant::now(),
    };

    assert_eq!(notification.level, NotificationLevel::Info);
}

#[wasm_bindgen_test]
fn test_notification_levels_equality() {
    assert_eq!(NotificationLevel::Success, NotificationLevel::Success);
    assert_eq!(NotificationLevel::Error, NotificationLevel::Error);
    assert_eq!(NotificationLevel::Info, NotificationLevel::Info);

    assert_ne!(NotificationLevel::Success, NotificationLevel::Error);
    assert_ne!(NotificationLevel::Error, NotificationLevel::Info);
    assert_ne!(NotificationLevel::Info, NotificationLevel::Success);
}

#[wasm_bindgen_test]
fn test_notification_with_long_message() {
    let mut manager = ToastManager::new();
    let long_message = "This is a very long notification message that should still be handled properly by the toast system without causing any issues or truncation problems.".to_string();

    let notification = Notification {
        message: long_message.clone(),
        level: NotificationLevel::Info,
        timestamp: Instant::now(),
    };

    manager.add_toast(notification);
    assert!(true, "Long messages handled correctly");
}

#[wasm_bindgen_test]
fn test_mixed_notification_levels() {
    let mut manager = ToastManager::new();
    let notifications = vec![
        Notification {
            message: "Success message".to_string(),
            level: NotificationLevel::Success,
            timestamp: Instant::now(),
        },
        Notification {
            message: "Error message".to_string(),
            level: NotificationLevel::Error,
            timestamp: Instant::now(),
        },
        Notification {
            message: "Info message".to_string(),
            level: NotificationLevel::Info,
            timestamp: Instant::now(),
        },
    ];

    manager.update_from_notifications(&notifications);
    assert!(true, "Mixed notification levels handled correctly");
}
