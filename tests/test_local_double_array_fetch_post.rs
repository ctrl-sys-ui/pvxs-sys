// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
mod test_pvxs_local_double_array_fetch_post {

    use pvxs_sys::{Server, NTScalarMetadataBuilder};

    #[test]
    fn test_pv_local_double_array_fetch_post() {
        // This test creates a local pv (loc:double:array) on a server and
        // tests server-side fetch() and post_double() operations.
        let initial_array = vec![3.14159, 2.71828, 1.61803];
        let name = "loc:double:array";
        let loc_srv = Server::start_isolated()
            .expect("Failed to create isolated server");

        // Create a double array PV and capture it for server-side operations
        loc_srv.create_pv_double_array(
            name, initial_array.clone(), NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:double:array");

        // Verify we can fetch the initial array value using server-side fetch
        match loc_srv.fetch_double_array(name) {
            Ok(fetched) => {
                assert_eq!(initial_array, fetched.value, "Initial array value mismatch, got {:?}, expected {:?}", fetched.value, initial_array);
            },
            Err(e) => panic!("Failed to fetch initial array value: {}", e),
        }

        // Test posting different double values and reading back using server-side operations
        let test_values = vec![0.0, -1.5, 2.71828, 1e-10, 1e10];
        loc_srv.post_double_array(name, test_values.clone())
            .expect("Failed to post double array");

        match loc_srv.fetch_double_array(name) {
            Ok(fetched) => {
                assert_eq!(test_values, fetched.value, "Posted array value mismatch, got {:?}, expected {:?}", fetched.value, test_values);
            },
            Err(e) => panic!("Failed to fetch posted array value: {}", e),
        }
    }

    #[test]
    fn test_pv_local_double_array_special_values() {
        
        // Test special double values using server-side post/fetch
        let mut special_values = vec![
            0.0, // Zero
            -0.0, // Negative zero
            std::f64::consts::PI, // PI
            std::f64::consts::E, // E
            f64::MAX, // Max
            f64::MIN, // Min
            f64::MIN_POSITIVE, // Min positive
            1e-308, // Very small
            1e308, // Very large
        ];

        // Test local handling of special floating point values using server-side operations
        let name = "loc:double:special";
        let loc_srv = Server::start_isolated()
            .expect("Failed to create isolated server");

        loc_srv.create_pv_double_array(name, special_values.clone(), NTScalarMetadataBuilder::new())
            .expect("Failed to create pv:double:special");
        
        match loc_srv.fetch_double_array(name) {
            Ok(fetched) => {
                assert_eq!(special_values, fetched.value, "Special values array mismatch, got {:?}, expected {:?}", fetched.value, special_values);
            },
            Err(e) => panic!("Failed to fetch special values array: {}", e),
        }

        // Add  infinity to the array and test
        special_values.push(f64::INFINITY);
        loc_srv.post_double_array(name, special_values.clone())
            .expect("Failed to post double array with infinity");
        match loc_srv.fetch_double_array(name) {
            Ok(fetched) => {
                assert_eq!(special_values, fetched.value, "Special values array with infinity mismatch, got {:?}, expected {:?}", fetched.value, special_values);
            },  
            Err(e) => panic!("Failed to fetch special values array with infinity: {}", e),
        }

        // Add negative infinity to the array and test
        special_values.push(f64::NEG_INFINITY);
        loc_srv.post_double_array(name, special_values.clone())
            .expect("Failed to post double array with negative infinity");
        match loc_srv.fetch_double_array(name) {
            Ok(fetched) => {
                assert_eq!(special_values, fetched.value, "Special values array with negative infinity mismatch, got {:?}, expected {:?}", fetched.value, special_values);
            },  
            Err(e) => panic!("Failed to fetch special values array with negative infinity: {}", e),
        }

        // Add NaN to the array and test
        special_values.push(f64::NAN);
        loc_srv.post_double_array(name, special_values.clone())
            .expect("Failed to post double array with NaN");
        match loc_srv.fetch_double_array(name) {
            Ok(fetched) => {
                assert_eq!(special_values.len(), fetched.value.len(), "Special values array length mismatch");
                for (i, (expected, actual)) in special_values.iter().zip(fetched.value.iter()).enumerate() {
                    if expected.is_nan() {
                        assert!(actual.is_nan(), "Element {} should be NaN, got {}", i, actual);
                    } else {
                        assert_eq!(expected, actual, "Element {} mismatch: expected {}, got {}", i, expected, actual);
                    }
                }
            },  
            Err(e) => panic!("Failed to fetch special values array with NaN: {}", e),
        }
    }
}
