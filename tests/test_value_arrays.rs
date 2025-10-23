// Tests for array field accessors

use epics_pvxs_sys::*;

#[test]
#[ignore] // Requires running EPICS server with array PVs
fn test_get_field_double_array() {
    // Example of how to use the double array accessor
    let mut ctx = Context::from_env().expect("Failed to create context");
    let value = ctx.get("test:double:array", 5.0).expect("Failed to get PV");
    
    match value.get_field_double_array("value") {
        Ok(arr) => {
            println!("Double array length: {}", arr.len());
            for (i, val) in arr.iter().enumerate() {
                println!("  [{}] = {}", i, val);
            }
        }
        Err(e) => println!("Error getting double array: {}", e),
    }
}

#[test]
#[ignore] // Requires running EPICS server with array PVs
fn test_get_field_int32_array() {
    // Example of how to use the int32 array accessor
    let mut ctx = Context::from_env().expect("Failed to create context");
    let value = ctx.get("test:int:array", 5.0).expect("Failed to get PV");
    
    match value.get_field_int32_array("value") {
        Ok(arr) => {
            println!("Int32 array length: {}", arr.len());
            for (i, val) in arr.iter().enumerate() {
                println!("  [{}] = {}", i, val);
            }
        }
        Err(e) => println!("Error getting int32 array: {}", e),
    }
}

#[test]
#[ignore] // Requires running EPICS server with array PVs
fn test_get_field_string_array() {
    // Example of how to use the string array accessor
    // This is commonly used for enum choices in NTEnum types
    let mut ctx = Context::from_env().expect("Failed to create context");
    let value = ctx.get("test:enum:pv", 5.0).expect("Failed to get PV");
    
    match value.get_field_string_array("value.choices") {
        Ok(arr) => {
            println!("String array (enum choices) length: {}", arr.len());
            for (i, val) in arr.iter().enumerate() {
                println!("  [{}] = '{}'", i, val);
            }
        }
        Err(e) => println!("Error getting string array: {}", e),
    }
}

#[test]
#[ignore] // Requires running EPICS server with array PVs
fn test_get_field_enum_array() {
    // Example of how to use the enum array accessor
    let mut ctx = Context::from_env().expect("Failed to create context");
    let value = ctx.get("test:enum:array", 5.0).expect("Failed to get PV");
    
    match value.get_field_enum_array("value") {
        Ok(arr) => {
            println!("Enum array length: {}", arr.len());
            for (i, val) in arr.iter().enumerate() {
                println!("  [{}] = {}", i, val);
            }
        }
        Err(e) => println!("Error getting enum array: {}", e),
    }
}

#[test]
#[ignore] // Requires running EPICS server
fn test_ntenum_with_choices() {
    // Example of working with NTEnum that has both value and choices
    let mut ctx = Context::from_env().expect("Failed to create context");
    let value = ctx.get("test:enum:pv", 5.0).expect("Failed to get PV");
    
    // Get the current enum index
    match value.get_field_enum("value.index") {
        Ok(index) => {
            println!("Current enum index: {}", index);
            
            // Get the choices array
            match value.get_field_string_array("value.choices") {
                Ok(choices) => {
                    if (index as usize) < choices.len() {
                        println!("Current enum value: '{}'", choices[index as usize]);
                        
                        println!("All choices:");
                        for (i, choice) in choices.iter().enumerate() {
                            let marker = if i == index as usize { " <--" } else { "" };
                            println!("  [{}] = '{}'{}", i, choice, marker);
                        }
                    }
                }
                Err(e) => println!("Error getting choices: {}", e),
            }
        }
        Err(e) => println!("Error getting enum index: {}", e),
    }
}
