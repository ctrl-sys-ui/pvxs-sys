// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use cxx::UniquePtr;
use std::fmt;

use crate::{bridge, Result};

/// A PVAccess value container
/// 
/// Represents a structured data value returned from PVXS operations.
/// Values have a hierarchical structure with named fields.
/// 
/// # Field Access
/// 
/// Values are accessed by field name. Common fields include:
/// - `"value"`: The primary data value
/// - `"alarm.severity"`: Alarm severity level
/// - `"alarm.status"`: Alarm status code
/// - `"timeStamp.secondsPastEpoch"`: Timestamp seconds
/// 
/// # Example
/// 
/// ```no_run
/// # use pvxs_sys::{Context, Value};
/// # let mut ctx = Context::from_env().unwrap();
/// let value: Value = ctx.get("my:pv:name", 5.0).unwrap();
/// 
/// // Access different field types
/// let v = value.get_field_double("value").unwrap();
/// let severity = value.get_field_int32("alarm.severity").unwrap();
/// ```
pub struct Value {
    pub(crate) inner: UniquePtr<bridge::ValueWrapper>,
}

impl Value {
    /// Check if this value is valid
    /// 
    /// Returns `false` if the value is empty or uninitialized.
    pub fn is_valid(&self) -> bool {
        bridge::value_is_valid(&self.inner)
    }
    
    /// Get a field value as a double
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to a double.
    pub fn get_field_double(&self, field_name: &str) -> Result<f64> {
        Ok(bridge::value_get_field_double(&self.inner, field_name.to_string())?)
    }
    
    /// Get a field value as an i32
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to an i32.
    pub fn get_field_int32(&self, field_name: &str) -> Result<i32> {
        Ok(bridge::value_get_field_int32(&self.inner, field_name.to_string())?)
    }
    
    /// Get a field value as a String
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to a string.
    pub fn get_field_string(&self, field_name: &str) -> Result<String> {
        Ok(bridge::value_get_field_string(&self.inner, field_name.to_string())?)
    }

    /// Get a field value as a enum
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to a enum.
    pub fn get_field_enum(&self, field_name: &str) -> Result<i16> {
        Ok(bridge::value_get_field_enum(&self.inner, field_name.to_string())?)
    }

    /// Get a field value as an array of doubles
    /// 
    /// Extracts a field containing an array of double-precision floating point values.
    /// Commonly used for waveform data, measurement arrays, or multi-point setpoints.
    /// 
    /// # Arguments
    /// 
    /// * `field_name` - The field path (e.g., "value", "waveform.data")
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to an array of doubles.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let value = ctx.get("waveform:double:pv", 5.0).unwrap();
    /// let array = value.get_field_double_array("value").unwrap();
    /// println!("Double array length: {}", array.len());
    /// for (i, val) in array.iter().enumerate().take(5) {
    ///     println!("  [{}] = {}", i, val);
    /// }
    /// ```
    pub fn get_field_double_array(&self, field_name: &str) -> Result<Vec<f64>> {
        Ok(bridge::value_get_field_double_array(&self.inner, field_name.to_string())?)
    }

    /// Get a field value as an array of int32
    /// 
    /// Extracts a field containing an array of 32-bit signed integers.
    /// Often used for status arrays, configuration parameters, or indexed data.
    /// 
    /// # Arguments
    /// 
    /// * `field_name` - The field path (e.g., "value", "status.codes")
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to an array of int32.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// let value = ctx.get("array:int32:pv", 5.0).unwrap();
    /// let array = value.get_field_int32_array("value").unwrap();
    /// println!("Int32 array length: {}", array.len());
    /// for (i, val) in array.iter().enumerate().take(5) {
    ///     println!("  [{}] = {}", i, val);
    /// }
    /// ```
    pub fn get_field_int32_array(&self, field_name: &str) -> Result<Vec<i32>> {
        Ok(bridge::value_get_field_int32_array(&self.inner, field_name.to_string())?)
    }

    /// Get a field value as an array of strings
    /// 
    /// Extracts a field containing an array of string values.
    /// Commonly used for enum choices, device names, status messages, or text lists.
    /// 
    /// # Arguments
    /// 
    /// * `field_name` - The field path (e.g., "value.choices", "devices.names")
    /// 
    /// # Errors
    /// 
    /// Returns an error if the field doesn't exist or cannot be
    /// converted to an array of strings.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use pvxs_sys::Context;
    /// # let mut ctx = Context::from_env().unwrap();
    /// // Get enum choices for an NTEnum PV
    /// let value = ctx.get("enum:pv", 5.0).unwrap();
    /// let choices = value.get_field_string_array("value.choices").unwrap();
    /// println!("Available choices:");
    /// for (i, choice) in choices.iter().enumerate() {
    ///     println!("  [{}] = '{}'", i, choice);
    /// }
    /// ```
    pub fn get_field_string_array(&self, field_name: &str) -> Result<Vec<String>> {
        Ok(bridge::value_get_field_string_array(&self.inner, field_name.to_string())?)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", bridge::value_to_string(&self.inner))
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Value")
            .field("data", &bridge::value_to_string(&self.inner))
            .finish()
    }
}
