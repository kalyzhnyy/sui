// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! This pass verifies necessary properties for Move Objects, i.e. structs with the `key` ability.
//! The properties checked are
//! - The first field is named "id"
//! - The first field has type `sui::id::VersionedID`

use crate::verification_failure;
use move_binary_format::{
    access::ModuleAccess,
    binary_views::BinaryIndexedView,
    file_format::{CompiledModule, SignatureToken},
};
use sui_types::{
    error::ExecutionError,
    fp_ensure,
    id::{ID_MODULE_NAME, VERSIONED_ID_STRUCT_NAME},
    SUI_FRAMEWORK_ADDRESS,
};

pub fn verify_module(module: &CompiledModule) -> Result<(), ExecutionError> {
    verify_key_structs(module)
}

fn verify_key_structs(module: &CompiledModule) -> Result<(), ExecutionError> {
    let view = BinaryIndexedView::Module(module);
    let struct_defs = &module.struct_defs;
    for def in struct_defs {
        let handle = module.struct_handle_at(def.struct_handle);
        if !handle.abilities.has_key() {
            continue;
        }
        let name = view.identifier_at(handle.name);

        // Check that the first field of the struct must be named "id".
        let first_field = match def.field(0) {
            Some(field) => field,
            None => {
                return Err(verification_failure(format!(
                    "First field of struct {} must be 'id', no field found",
                    name
                )))
            }
        };
        let first_field_name = view.identifier_at(first_field.name).as_str();
        if first_field_name != "id" {
            return Err(verification_failure(format!(
                "First field of struct {} must be 'id', {} found",
                name, first_field_name
            )));
        }
        // Check that the "id" field must have a struct type.
        let id_field_type = &first_field.signature.0;
        let id_field_type = match id_field_type {
            SignatureToken::Struct(struct_type) => struct_type,
            _ => {
                return Err(verification_failure(format!(
                    "First field of struct {} must be of ID type, {:?} type found",
                    name, id_field_type
                )))
            }
        };
        // Chech that the struct type for "id" field must be SUI_FRAMEWORK_ADDRESS::ID::ID.
        let id_type_struct = module.struct_handle_at(*id_field_type);
        let id_type_struct_name = view.identifier_at(id_type_struct.name);
        let id_type_module = module.module_handle_at(id_type_struct.module);
        let id_type_module_address = module.address_identifier_at(id_type_module.address);
        let id_type_module_name = module.identifier_at(id_type_module.name);
        fp_ensure!(
            id_type_struct_name == VERSIONED_ID_STRUCT_NAME
                && id_type_module_address == &SUI_FRAMEWORK_ADDRESS
                && id_type_module_name == ID_MODULE_NAME,
            verification_failure(format!(
                "First field of struct {} must be of type {}::id::VersionedID, {}::{}::{} type found",
                name,
                SUI_FRAMEWORK_ADDRESS,
                id_type_module_address,
                id_type_module_name,
                id_type_struct_name
            )));
    }
    Ok(())
}
