// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
mod test_pvxs_local_double_fetch_post {
    use pvxs_sys::{NTScalarMetadataBuilder, Server};

    #[test]
    fn test_pv_local_double_fetch_post() {
        // This test creates a local pv (loc:double) on a server and gets
        // and sets the value on server side.
        let initial_value = 3.14159;
        let name = "loc:double";
        let loc_srv = Server::start_isolated().expect("Failed to create isolated server");

        loc_srv
            .create_pv_double(name, initial_value, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:double");

        // Do a server side fetch to verify initial value
        match loc_srv.fetch_double(name) {
            Ok(fetched) => assert_eq!(fetched.value, initial_value, "Initial value mismatch"),
            Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
        }

        // Post an int32 to simulate type mismatch... negative test
        match loc_srv.post_int32(name, 42) {
            Ok(_) => assert!(
                false,
                "Expected error when posting int32 to double pv, but got Ok"
            ),
            Err(e) => assert!(
                format!("{}", e).contains("type mismatch"),
                "Expected type mismatch error, but got: {:?}",
                e
            ),
        }

        // Fetch again to verify the converted value has not changed due to failed post
        match loc_srv.fetch_double(name) {
            Ok(fetched) => assert_eq!(
                fetched.value, initial_value,
                "Value mismatch after posting int32"
            ),
            Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
        }

        // Post a string to simulate value being invalid... negative test
        match loc_srv.post_string(name, "not_a_number") {
            Ok(_) => assert!(
                false,
                "Expected error when posting invalid string to double pv, but got Ok"
            ),
            Err(_) => assert!(true), // Expected error
        }

        // Fetch again to verify the converted value has not changed due to failed post
        match loc_srv.fetch_double(name) {
            Ok(fetched) => assert_eq!(
                fetched.value, initial_value,
                "Value mismatch after posting string"
            ),
            Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
        }

        // Now set a new value and do a server side post
        let new_value = 2.71828;
        match loc_srv.post_double(name, new_value) {
            Ok(_) => (),
            Err(e) => assert!(false, "Failed to post new value: {:?}", e),
        }

        // Fetch again to verify the new value
        match loc_srv.fetch_double(name) {
            Ok(fetched) => assert_eq!(
                fetched.value, new_value,
                "Value mismatch after posting new double"
            ),
            Err(e) => assert!(false, "Failed to fetch value: {:?}", e),
        }
    }

    #[test]
    fn test_pv_local_double_fetch_post_with_error_propagation(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let initial_value = 123.456;
        let name = "loc:double";
        // This test verifies that errors in get/set operations are properly propagated.
        let loc_srv = Server::start_isolated()?;

        loc_srv.create_pv_double(name, initial_value, NTScalarMetadataBuilder::new())?;

        // Intentionally cause an error by trying to post an invalid value
        match loc_srv.post_string(name, "invalid_double") {
            Ok(_) => assert!(
                false,
                "Expected error when posting invalid value, but got Ok"
            ),
            Err(_) => assert!(true), // Expected error
        }

        // Verify that fetching still works after the error
        let fetched_value = loc_srv.fetch_double(name)?;
        assert_eq!(
            fetched_value.value, initial_value,
            "Value should remain unchanged after failed post"
        );

        // Now post a valid value and verify
        let new_value = 987.654;
        loc_srv.post_double(name, new_value)?;
        let fetched_value = loc_srv.fetch_double(name)?;
        assert_eq!(
            fetched_value.value, new_value,
            "Value mismatch after posting new double"
        );

        Ok(())
    }

    #[test]
    fn test_pv_local_double_special_values() {
        let name = "loc:double";
        // Test handling of special floating point values
        let loc_srv = Server::start_isolated().expect("Failed to create isolated server");

        loc_srv
            .create_pv_double(name, 0.0, NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:double");

        // Test positive infinity
        match loc_srv.post_double(name, f64::INFINITY) {
            Ok(_) => {
                let value = loc_srv.fetch_double(name).unwrap();
                assert!(value.value.is_infinite());
            }
            Err(e) => assert!(false, "Server may not support infinity: {:?}", e),
        }

        // Test negative infinity
        match loc_srv.post_double(name, f64::NEG_INFINITY) {
            Ok(_) => {
                let value = loc_srv.fetch_double(name).unwrap();
                assert!(value.value.is_infinite());
            }
            Err(e) => assert!(false, "Server may not support negative infinity: {:?}", e),
        }

        // Test NaN (may not be supported by EPICS)
        match loc_srv.post_double(name, f64::NAN) {
            Ok(_) => {
                // Note: NaN comparisons always return false, so we can't use assert_eq
                assert!(
                    loc_srv.fetch_double(name).is_ok(),
                    "NaN value did not post successfully"
                );
            }
            Err(e) => assert!(false, "Server may not support NaN: {:?}", e),
        }

        // Test very large and very small numbers
        loc_srv
            .post_double(name, f64::MAX)
            .expect("Failed to post max value");
        let value = loc_srv.fetch_double(name).unwrap();
        assert_eq!(value.value, f64::MAX);

        loc_srv
            .post_double(name, f64::MIN)
            .expect("Failed to post min value");
        let value = loc_srv.fetch_double(name).unwrap();
        assert_eq!(value.value, f64::MIN);
    }
}
