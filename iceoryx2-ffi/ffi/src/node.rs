// Copyright (c) 2024 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(non_camel_case_types)]

use crate::{
    iox2_callback_progression_e, iox2_config_ptr, iox2_node_name_ptr, iox2_service_builder_h,
    iox2_service_builder_t, iox2_service_name_h, iox2_service_type_e, HandleToType, IntoCInt,
    IOX2_OK,
};

use iceoryx2::node::{NodeId, NodeListFailure, NodeView};
use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::ffi::{c_int, c_void};
use core::mem::ManuallyDrop;

// BEGIN type definition

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_node_list_failure_e {
    INSUFFICIENT_PERMISSIONS = IOX2_OK as isize + 1,
    INTERRUPT,
    INTERNAL_ERROR,
}

impl IntoCInt for NodeListFailure {
    fn into_c_int(self) -> c_int {
        (match self {
            NodeListFailure::InsufficientPermissions => {
                iox2_node_list_failure_e::INSUFFICIENT_PERMISSIONS
            }
            NodeListFailure::Interrupt => iox2_node_list_failure_e::INTERRUPT,
            NodeListFailure::InternalError => iox2_node_list_failure_e::INTERNAL_ERROR,
        }) as c_int
    }
}

pub(crate) union NodeUnion {
    ipc: ManuallyDrop<Node<zero_copy::Service>>,
    local: ManuallyDrop<Node<process_local::Service>>,
}

impl NodeUnion {
    pub(crate) fn new_ipc(node: Node<zero_copy::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(node),
        }
    }
    pub(crate) fn new_local(node: Node<process_local::Service>) -> Self {
        Self {
            local: ManuallyDrop::new(node),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<NodeUnion>
pub struct iox2_node_storage_t {
    internal: [u8; 16], // magic number obtained with size_of::<Option<NodeUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(NodeUnion)]
pub struct iox2_node_t {
    pub(crate) service_type: iox2_service_type_e,
    pub(crate) value: iox2_node_storage_t,
    pub(crate) deleter: fn(*mut iox2_node_t),
}

impl iox2_node_t {
    pub(crate) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: NodeUnion,
        deleter: fn(*mut iox2_node_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_name_h_t;
/// The owning handle for `iox2_node_t`. Passing the handle to an function transfers the ownership.
pub type iox2_node_h = *mut iox2_name_h_t;

pub struct iox2_noderef_h_t;
/// The non-owning handle for `iox2_node_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_node_ref_h = *mut iox2_noderef_h_t;

impl HandleToType for iox2_node_h {
    type Target = *mut iox2_node_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_node_ref_h {
    type Target = *mut iox2_node_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_node_state_e {
    ALIVE,
    DEAD,
    INACCESSIBLE,
    UNDEFINED,
}

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `NodeId`
pub type iox2_node_id_ptr = *const NodeId;
/// The mutable pointer to the underlying `NodeId`
pub type iox2_node_id_mut_ptr = *mut NodeId;

/// An alias to a `void *` which can be used to pass arbitrary data to the callback
pub type iox2_node_list_callback_context = *mut c_void;

/// The callback for [`iox2_node_list`]
///
/// # Arguments
///
/// * [`iox2_node_state_e`]
/// * [`iox2_node_id_ptr`]
/// * [`iox2_node_name_ptr`](crate::iox2_node_name_ptr) -> `NULL` for `iox2_node_state_e::INACCESSIBLE` and `iox2_node_state_e::UNDEFINED`
/// * [`iox2_config_ptr`](crate::iox2_config_ptr) -> `NULL` for `iox2_node_state_e::INACCESSIBLE` and `iox2_node_state_e::UNDEFINED`
/// * [`iox2_node_list_callback_context`] -> provided by the user to [`iox2_node_list`] and can be `NULL`
///
/// Returns a [`iox2_callback_progression_e`](crate::iox2_callback_progression_e)
pub type iox2_node_list_callback = extern "C" fn(
    iox2_node_state_e,
    iox2_node_id_ptr,
    iox2_node_name_ptr,
    iox2_config_ptr,
    iox2_node_list_callback_context,
) -> iox2_callback_progression_e;

// END type definition

// BEGIN C API

/// Returns the [`iox2_node_name_ptr`](crate::iox2_node_name_ptr), an immutable pointer to the node name.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name(node_handle: iox2_node_h) -> iox2_node_name_ptr {
    debug_assert!(!node_handle.is_null());

    let node = &mut *node_handle.as_type();

    match node.service_type {
        iox2_service_type_e::IPC => node.value.as_ref().ipc.name(),
        iox2_service_type_e::LOCAL => node.value.as_ref().local.name(),
    }
}

/// Returns the [`iox2_config_ptr`](crate::iox2_config_ptr), an immutable pointer to the config.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_config(node_handle: iox2_node_h) -> iox2_config_ptr {
    debug_assert!(!node_handle.is_null());

    let node = &mut *node_handle.as_type();

    match node.service_type {
        iox2_service_type_e::IPC => node.value.as_ref().ipc.config(),
        iox2_service_type_e::LOCAL => node.value.as_ref().local.config(),
    }
}

/// Returns the [`iox2_node_id_ptr`](crate::iox2_node_id_ptr), an immutable pointer to the node id.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_id(node_handle: iox2_node_h) -> iox2_node_id_ptr {
    debug_assert!(!node_handle.is_null());
    todo!() // TODO: [#210] implement
}

fn iox2_node_list_impl<S: Service>(
    node_state: &NodeState<S>,
    callback: iox2_node_list_callback,
    callback_ctx: iox2_node_list_callback_context,
) -> CallbackProgression {
    match node_state {
        NodeState::Alive(alive_node_view) => {
            let (node_name, config) = alive_node_view
                .details()
                .as_ref()
                .map(|view| (view.name() as _, view.config() as _))
                .unwrap_or((std::ptr::null(), std::ptr::null()));
            callback(
                iox2_node_state_e::ALIVE,
                alive_node_view.id(),
                node_name,
                config,
                callback_ctx,
            )
            .into()
        }
        NodeState::Dead(dead_node_view) => {
            let (node_name, config) = dead_node_view
                .details()
                .as_ref()
                .map(|view| (view.name() as _, view.config() as _))
                .unwrap_or((std::ptr::null(), std::ptr::null()));
            callback(
                iox2_node_state_e::DEAD,
                dead_node_view.id(),
                node_name,
                config,
                callback_ctx,
            )
            .into()
        }
        NodeState::Inaccessible(ref node_id) => callback(
            iox2_node_state_e::INACCESSIBLE,
            node_id,
            std::ptr::null(),
            std::ptr::null(),
            callback_ctx,
        )
        .into(),
        NodeState::Undefined(ref node_id) => callback(
            iox2_node_state_e::UNDEFINED,
            node_id,
            std::ptr::null(),
            std::ptr::null(),
            callback_ctx,
        )
        .into(),
    }
}

/// Calls the callback repeatedly with an [`iox2_node_state_e`], [`iox2_node_id_ptr`], [´iox2_node_name_ptr´] and [`iox2_config_ptr`] for
/// all [`Node`](iceoryx2::node::Node)s in the system under a given [`Config`](iceoryx2::config::Config).
///
/// # Arguments
///
/// * `service_type` - A [`iox2_service_type_e`]
/// * `config_ptr` - A valid [`iox2_config_ptr`](crate::iox2_config_ptr)
/// * `callback` - A valid callback with [`iox2_node_list_callback`} signature
/// * `callback_ctx` - An optional callback context [`iox2_node_list_callback_context`} to e.g. store information across callback iterations
///
/// Returns IOX2_OK on success, an [`iox2_node_list_failure_e`] otherwise.
///
/// # Safety
///
/// * The `config_ptr` must be valid and obtained by ether [`iox2_node_config`] or [`iox2_config_global_config`](crate::iox2_config_global_config)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_list(
    service_type: iox2_service_type_e,
    config_ptr: iox2_config_ptr,
    callback: iox2_node_list_callback,
    callback_ctx: iox2_node_list_callback_context,
) -> c_int {
    debug_assert!(!config_ptr.is_null());

    let config = &*config_ptr;

    let list_result = match service_type {
        iox2_service_type_e::IPC => Node::<zero_copy::Service>::list(config, |node_state| {
            iox2_node_list_impl(&node_state, callback, callback_ctx)
        }),
        iox2_service_type_e::LOCAL => Node::<process_local::Service>::list(config, |node_state| {
            iox2_node_list_impl(&node_state, callback, callback_ctx)
        }),
    };

    match list_result {
        Ok(_) => IOX2_OK,
        Err(e) => e.into_c_int(),
    }
}

#[no_mangle]
pub extern "C" fn iox2_service_name_new() {
    todo!() // TODO: [#210] implement
}
/// Instantiates a [`iox2_service_builder_h`] for a service with the provided name.
///
/// # Safety
///
/// * The `node_handle` must be valid and obtained by [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
/// * The `service_name_handle` must be valid and obtained by [`iox2_service_name_new`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_service_builder(
    node_handle: iox2_node_h,
    _service_builder: *mut iox2_service_builder_t,
    service_name_handle: iox2_service_name_h,
) -> iox2_service_builder_h {
    debug_assert!(!node_handle.is_null());
    debug_assert!(!service_name_handle.is_null());
    todo!() // TODO: [#210] implement
}

/// This function needs to be called to destroy the node!
///
/// # Arguments
///
/// * `node_handle` - A valid [`iox2_node_h`]
///
/// # Safety
///
/// * The `node_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_node_t`] can be re-used with a call to [`iox2_node_builder_create`](crate::iox2_node_builder_create)!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_drop(node_handle: iox2_node_h) {
    debug_assert!(!node_handle.is_null());

    let node = &mut *node_handle.as_type();

    match node.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut node.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut node.value.as_mut().local);
        }
    }
    (node.deleter)(node);
}

// END C API

#[cfg(test)]
mod test {
    use crate::*;
    use iceoryx2_bb_testing::assert_that;

    use core::{slice, str};

    fn create_sut_node() -> iox2_node_h {
        unsafe {
            let node_builder_handle = iox2_node_builder_new(std::ptr::null_mut());

            let mut node_name_handle = std::ptr::null_mut();
            let node_name = "hypnotoad";
            let ret_val = iox2_node_name_new(
                std::ptr::null_mut(),
                node_name.as_ptr() as *const _,
                node_name.len() as _,
                &mut node_name_handle,
            );
            assert_that!(ret_val, eq(IOX2_OK));
            iox2_node_builder_set_name(
                iox2_cast_node_builder_ref_h(node_builder_handle),
                node_name_handle,
            );

            let mut node_handle: iox2_node_h = std::ptr::null_mut();
            let ret_val = iox2_node_builder_create(
                node_builder_handle,
                std::ptr::null_mut(),
                iox2_service_type_e::IPC,
                &mut node_handle as *mut iox2_node_h,
            );

            assert_that!(ret_val, eq(IOX2_OK));

            node_handle
        }
    }

    #[test]
    fn basic_node_api_test() {
        unsafe {
            let node_handle = create_sut_node();

            assert_that!(node_handle, ne(std::ptr::null_mut()));

            iox2_node_drop(node_handle);
        }
    }

    #[test]
    fn basic_node_config_test() {
        unsafe {
            let node_handle = create_sut_node();

            let expected_config = Config::global_config();

            let config = iox2_node_config(node_handle);

            assert_that!(*(config as *const Config), eq(*expected_config));

            iox2_node_drop(node_handle);
        }
    }

    #[test]
    fn basic_node_name_test() -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let node_handle = create_sut_node();

            let node_name = iox2_node_name(node_handle);

            let mut node_name_len = 0;
            let node_name_c_str = iox2_node_name_as_c_str(node_name, &mut node_name_len);

            let slice = slice::from_raw_parts(node_name_c_str as *const _, node_name_len as _);
            let node_name_str = str::from_utf8(slice)?;

            assert_that!(node_name_str, eq("hypnotoad"));

            iox2_node_drop(node_handle);

            Ok(())
        }
    }

    #[derive(Default)]
    struct NodeListCtx {
        alive: u64,
        dead: u64,
        inaccessible: u64,
        undefined: u64,
    }

    extern "C" fn node_list_callback(
        node_state: iox2_node_state_e,
        _node_id_ptr: iox2_node_id_ptr,
        _node_name_ptr: iox2_node_name_ptr,
        _config_ptr: iox2_config_ptr,
        ctx: iox2_node_list_callback_context,
    ) -> iox2_callback_progression_e {
        let ctx = unsafe { &mut *(ctx as *mut NodeListCtx) };

        match node_state {
            iox2_node_state_e::ALIVE => {
                ctx.alive += 1;
            }
            iox2_node_state_e::DEAD => {
                ctx.dead += 1;
            }
            iox2_node_state_e::INACCESSIBLE => {
                ctx.inaccessible += 1;
            }
            iox2_node_state_e::UNDEFINED => {
                ctx.undefined += 1;
            }
        }

        iox2_callback_progression_e::CONTINUE
    }

    #[test]
    fn basic_node_list_test() {
        unsafe {
            let mut ctx = NodeListCtx::default();
            let node_handle = create_sut_node();
            let config = iox2_node_config(node_handle);

            let ret_val = iox2_node_list(
                iox2_service_type_e::IPC,
                config,
                node_list_callback,
                &mut ctx as *mut _ as *mut _,
            );

            iox2_node_drop(node_handle);

            assert_that!(ret_val, eq(IOX2_OK));

            assert_that!(ctx.alive, eq(1));
            assert_that!(ctx.dead, eq(0));
            assert_that!(ctx.inaccessible, eq(0));
            assert_that!(ctx.undefined, eq(0));
        }
    }
}
