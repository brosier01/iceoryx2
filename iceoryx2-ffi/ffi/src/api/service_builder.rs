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

use crate::api::{iox2_service_type_e, AssertNonNullHandle, HandleToType};

use iceoryx2::prelude::*;
use iceoryx2::service::builder::publish_subscribe::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2::service::builder::{
    event::Builder as ServiceBuilderEvent, publish_subscribe::Builder as ServiceBuilderPubSub,
    Builder as ServiceBuilderBase,
};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use core::mem::ManuallyDrop;
use core::mem::MaybeUninit;

// BEGIN types definition

pub(super) type UserHeaderFfi = CustomHeaderMarker;
pub(super) type PayloadFfi = [CustomPayloadMarker];
pub(super) type UninitPayloadFfi = [MaybeUninit<CustomPayloadMarker>];

pub(super) union ServiceBuilderUnionNested<S: Service> {
    pub(super) base: ManuallyDrop<ServiceBuilderBase<S>>,
    pub(super) event: ManuallyDrop<ServiceBuilderEvent<S>>,
    pub(super) pub_sub: ManuallyDrop<ServiceBuilderPubSub<PayloadFfi, UserHeaderFfi, S>>,
}

pub(super) union ServiceBuilderUnion {
    pub(super) ipc: ManuallyDrop<ServiceBuilderUnionNested<ipc::Service>>,
    pub(super) local: ManuallyDrop<ServiceBuilderUnionNested<local::Service>>,
}

impl ServiceBuilderUnion {
    pub(super) fn new_ipc_base(service_builder: ServiceBuilderBase<ipc::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(ServiceBuilderUnionNested::<ipc::Service> {
                base: ManuallyDrop::new(service_builder),
            }),
        }
    }

    pub(super) fn new_ipc_event(service_builder: ServiceBuilderEvent<ipc::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(ServiceBuilderUnionNested::<ipc::Service> {
                event: ManuallyDrop::new(service_builder),
            }),
        }
    }

    pub(super) fn new_ipc_pub_sub(
        service_builder: ServiceBuilderPubSub<PayloadFfi, UserHeaderFfi, ipc::Service>,
    ) -> Self {
        Self {
            ipc: ManuallyDrop::new(ServiceBuilderUnionNested::<ipc::Service> {
                pub_sub: ManuallyDrop::new(service_builder),
            }),
        }
    }

    pub(super) fn new_local_base(service_builder: ServiceBuilderBase<local::Service>) -> Self {
        Self {
            local: ManuallyDrop::new(ServiceBuilderUnionNested::<local::Service> {
                base: ManuallyDrop::new(service_builder),
            }),
        }
    }

    pub(super) fn new_local_event(service_builder: ServiceBuilderEvent<local::Service>) -> Self {
        Self {
            local: ManuallyDrop::new(ServiceBuilderUnionNested::<local::Service> {
                event: ManuallyDrop::new(service_builder),
            }),
        }
    }

    pub(super) fn new_local_pub_sub(
        service_builder: ServiceBuilderPubSub<PayloadFfi, UserHeaderFfi, local::Service>,
    ) -> Self {
        Self {
            local: ManuallyDrop::new(ServiceBuilderUnionNested::<local::Service> {
                pub_sub: ManuallyDrop::new(service_builder),
            }),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<ServiceBuilderUnion>
pub struct iox2_service_builder_storage_t {
    internal: [u8; 632], // magic number obtained with size_of::<Option<ServiceBuilderUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(ServiceBuilderUnion)]
pub struct iox2_service_builder_t {
    pub(super) service_type: iox2_service_type_e,
    pub(super) value: iox2_service_builder_storage_t,
    pub(super) deleter: fn(*mut iox2_service_builder_t),
}

impl iox2_service_builder_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: ServiceBuilderUnion,
        deleter: fn(*mut iox2_service_builder_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_service_builder_h_t;
/// The owning handle for `iox2_service_builder_t`. Passing the handle to an function transfers the ownership.
pub type iox2_service_builder_h = *mut iox2_service_builder_h_t;
/// The non-owning handle for `iox2_service_builder_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_service_builder_h_ref = *const iox2_service_builder_h;

pub struct iox2_service_builder_event_h_t;
/// The owning handle for `iox2_service_builder_t` which is already configured as event. Passing the handle to an function transfers the ownership.
pub type iox2_service_builder_event_h = *mut iox2_service_builder_event_h_t;
/// The non-owning handle for `iox2_service_builder_t` which is already configured as event. Passing the handle to an function does not transfers the ownership.
pub type iox2_service_builder_event_h_ref = *const iox2_service_builder_event_h;

pub struct iox2_service_builder_pub_sub_h_t;
/// The owning handle for `iox2_service_builder_t` which is already configured as event. Passing the handle to an function transfers the ownership.
pub type iox2_service_builder_pub_sub_h = *mut iox2_service_builder_pub_sub_h_t;
/// The non-owning handle for `iox2_service_builder_t` which is already configured as event. Passing the handle to an function does not transfers the ownership.
pub type iox2_service_builder_pub_sub_h_ref = *const iox2_service_builder_pub_sub_h;

impl AssertNonNullHandle for iox2_service_builder_event_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_service_builder_event_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl AssertNonNullHandle for iox2_service_builder_pub_sub_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_service_builder_pub_sub_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_service_builder_h {
    type Target = *mut iox2_service_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_service_builder_h_ref {
    type Target = *mut iox2_service_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

impl HandleToType for iox2_service_builder_event_h {
    type Target = *mut iox2_service_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_service_builder_event_h_ref {
    type Target = *mut iox2_service_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

impl HandleToType for iox2_service_builder_pub_sub_h {
    type Target = *mut iox2_service_builder_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_service_builder_pub_sub_h_ref {
    type Target = *mut iox2_service_builder_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}

// END type definition

// BEGIN C API

/// This function transform the [`iox2_service_builder_h`] to an event service builder.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_event_h`] obtained by [`iox2_node_service_builder`](crate::iox2_node_service_builder)
///
/// Returns a [`iox2_service_builder_event_h`] for the event service builder
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after this call; The corresponding `iox2_service_builder_t` is now owned by the returned handle.
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_event(
    service_builder_handle: iox2_service_builder_h,
) -> iox2_service_builder_event_h {
    debug_assert!(!service_builder_handle.is_null());

    let service_builders_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builders_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.base);
            service_builders_struct
                .set(ServiceBuilderUnion::new_ipc_event(service_builder.event()));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.base);
            service_builders_struct.set(ServiceBuilderUnion::new_local_event(
                service_builder.event(),
            ));
        }
    }

    service_builder_handle as *mut _ as _
}

/// This function transform the [`iox2_service_builder_h`] to a publish-subscribe service builder.
///
/// # Arguments
///
/// * `service_builder_handle` - Must be a valid [`iox2_service_builder_pub_sub_h`] obtained by [`iox2_node_service_builder`](crate::iox2_node_service_builder)
///
/// Returns a [`iox2_service_builder_pub_sub_h`] for the publish-subscribe service builder
///
/// # Safety
///
/// * The `service_builder_handle` is invalid after this call; The corresponding `iox2_service_builder_t` is now owned by the returned handle.
#[no_mangle]
pub unsafe extern "C" fn iox2_service_builder_pub_sub(
    service_builder_handle: iox2_service_builder_h,
) -> iox2_service_builder_pub_sub_h {
    debug_assert!(!service_builder_handle.is_null());

    let service_builders_struct = unsafe { &mut *service_builder_handle.as_type() };

    match service_builders_struct.service_type {
        iox2_service_type_e::IPC => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().ipc);

            let service_builder = ManuallyDrop::into_inner(service_builder.base);
            service_builders_struct.set(ServiceBuilderUnion::new_ipc_pub_sub(
                service_builder
                    .publish_subscribe::<PayloadFfi>()
                    .user_header::<UserHeaderFfi>(),
            ));
        }
        iox2_service_type_e::LOCAL => {
            let service_builder =
                ManuallyDrop::take(&mut service_builders_struct.value.as_mut().local);

            let service_builder = ManuallyDrop::into_inner(service_builder.base);
            service_builders_struct.set(ServiceBuilderUnion::new_local_pub_sub(
                service_builder
                    .publish_subscribe::<PayloadFfi>()
                    .user_header::<UserHeaderFfi>(),
            ));
        }
    }

    service_builder_handle as *mut _ as _
}

// END C API
