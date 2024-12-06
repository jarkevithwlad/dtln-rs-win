// C FFI bindings to tensorflowlite_c

// Copyright (C) 2020 Scott Lamb <slamb@slamb.org>
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libc::c_char;
use libc::c_void;

#[repr(C)]
pub struct TfLiteDelegate {
    _private: [u8; 0],
}
#[repr(C)]
pub struct TfLiteInterpreter {
    _private: [u8; 0],
}
#[repr(C)]
pub struct TfLiteInterpreterOptions {
    _private: [u8; 0],
}
#[repr(C)]
pub struct TfLiteModel {
    _private: [u8; 0],
}
#[repr(C)]
pub struct TfLiteTensor {
    _private: [u8; 0],
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum Type {
    NoType = 0,
    Float32 = 1,
    Int32 = 2,
    UInt8 = 3,
    Int64 = 4,
    String = 5,
    Bool = 6,
    Int16 = 7,
    Complex64 = 8,
    Int8 = 9,
    Float16 = 10,
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TfLiteStatus(libc::c_int);

extern "C" {
    pub fn TfLiteModelCreate(model_data: *const u8, model_size: usize) -> *mut TfLiteModel;
    pub fn TfLiteModelCreateFromFile(model_path: *const c_char) -> *mut TfLiteModel;
    pub fn TfLiteModelDelete(model: *const TfLiteModel);

    pub fn TfLiteInterpreterOptionsSetNumThreads(
        options: *mut TfLiteInterpreterOptions,
        num_threads: i32,
    );

    pub fn TfLiteInterpreterOptionsCreate() -> *mut TfLiteInterpreterOptions;
    pub fn TfLiteInterpreterOptionsDelete(interpreter: *mut TfLiteInterpreterOptions);
    pub fn TfLiteInterpreterOptionsAddDelegate(
        options: *mut TfLiteInterpreterOptions,
        delegate: *mut TfLiteDelegate,
    );

    pub fn TfLiteInterpreterCreate(
        model: *const TfLiteModel,
        options: *const TfLiteInterpreterOptions,
    ) -> *mut TfLiteInterpreter;
    pub fn TfLiteInterpreterDelete(interpreter: *mut TfLiteInterpreter);
    pub fn TfLiteInterpreterAllocateTensors(interpreter: *mut TfLiteInterpreter) -> TfLiteStatus;
    pub fn TfLiteInterpreterGetInputTensorCount(
        interpreter: *const TfLiteInterpreter,
    ) -> libc::c_int;
    pub fn TfLiteInterpreterGetInputTensor(
        interpreter: *const TfLiteInterpreter,
        input_index: i32,
    ) -> *mut TfLiteTensor;
    pub fn TfLiteInterpreterInvoke(interpreter: *mut TfLiteInterpreter) -> TfLiteStatus;
    pub fn TfLiteInterpreterGetOutputTensorCount(
        interpreter: *const TfLiteInterpreter,
    ) -> libc::c_int;
    pub fn TfLiteInterpreterGetOutputTensor(
        interpreter: *const TfLiteInterpreter,
        output_index: i32,
    ) -> *const TfLiteTensor;

    pub fn TfLiteTensorType(tensor: *const TfLiteTensor) -> Type;
    pub fn TfLiteTensorNumDims(tensor: *const TfLiteTensor) -> i32;
    pub fn TfLiteTensorDim(tensor: *const TfLiteTensor, dim_index: i32) -> i32;
    pub fn TfLiteTensorByteSize(tensor: *const TfLiteTensor) -> usize;
    pub fn TfLiteTensorData(tensor: *const TfLiteTensor) -> *mut u8;
    pub fn TfLiteTensorName(tensor: *const TfLiteTensor) -> *const c_char;
    pub fn TfLiteInterpreterResizeInputTensor(
        interpreter: *const TfLiteInterpreter,
        input_index: usize,
        input_data: *const c_void,
        input_data_size: usize,
    ) -> TfLiteStatus;

    // fn TfLiteTypeGetName(type_: Type) -> *const c_char;
}

impl TfLiteStatus {
    pub fn to_result(self) -> Result<()> {
        match self.0 {
            0 => Ok(()),
            _ => Err(anyhow::anyhow!("TfLiteStatus {}", self.0)),
        }
    }
}
