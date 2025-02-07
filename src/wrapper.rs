#![allow(dead_code)]
use crate::types::{dtrace_aggwalk_order, dtrace_status};
use crate::utils::{Error, self};
use ::core::ffi::c_int;
/// Represents a handle to a DTrace instance.
pub struct dtrace_hdl {
    handle: *mut crate::dtrace_hdl_t,
}

impl From<*mut crate::dtrace_hdl_t> for dtrace_hdl {
    fn from(value: *mut crate::dtrace_hdl_t) -> Self {
        Self { handle: value }
    }
}

impl Drop for dtrace_hdl {
    fn drop(&mut self) {
        unsafe {
            crate::dtrace_close(self.handle);
        }
    }
}

unsafe impl Send for dtrace_hdl {}
unsafe impl Sync for dtrace_hdl {}

impl dtrace_hdl {
    /* General Purpose APIs BEGIN */
    /// Opens a DTrace instance with the specified version and flags.
    ///
    /// # Arguments
    ///
    /// * `version` - The DTrace version to use, `DTRACE_VERSION`. Specifying any version other than the current version causes DTrace to fail.
    /// * `flags` - Flags to control the behavior of the DTrace instance. Available flags:
    ///     * `DTRACE_O_NODEV` - Do not attempt to open any DTrace devices.
    ///     * `DTRACE_O_NOSYS` - Do not attempt to enable any DTrace providers.
    ///     * `DTRACE_O_LP64` - Force DTrace to operate in 64-bit mode.
    ///     * `DTRACE_O_ILP32` - Force DTrace to operate in 32-bit mode.
    /// # Returns
    ///
    /// Returns a `Result` containing the `dtrace_hdl` handle if successful, or an error code if
    /// the DTrace instance could not be opened.
    pub fn dtrace_open(version: c_int, flags: c_int) -> Result<Self, Error> {
        let mut errp: c_int = 0;

        let handle = unsafe { crate::dtrace_open(version, flags, &mut errp) };

        if handle.is_null() {
            return Err(Error::from(errp));
        }

        Ok(handle.into())
    }

    /// Starts the execution of the program.
    ///
    /// This action enables the specified probes. After `dtrace_go` function is called, the probes start to generate data.
    /// # Returns
    ///
    /// * `Ok(())` - If the program execution is successful.
    /// * `Err(errno)` - If the program execution fails. The error number (`errno`) is returned.
    pub fn dtrace_go(&self) -> Result<(), Error> {
        match unsafe { crate::dtrace_go(self.handle) } {
            0 => Ok(()),
            _ => Err(Error::from(self)),
        }
    }

    /// Stops the DTrace data consumption.
    ///
    /// This function communicates to the kernel that this consumer no longer consumes data. The kernel disables any enabled probe and frees the memory for the buffers associated with this DTrace handle.
    ///
    /// If the consumer does not call the `dtrace_stop()` function, the kernel eventually performs the cleanup. The data gathering stops either when the `deadman` timer fires or when the DTrace device is closed. The buffers are freed when the device closes. The DTrace device closes either when the consumer calls the `dtrace_close()` function or when the consumer exits. It is best practice for the consumer to call the `dtrace_stop()` function.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the stop operation is successful.
    /// * `Err(String)` - If the stop operation fails. The error message is returned.
    pub fn dtrace_stop(&self) -> Result<(), Error> {
        match unsafe { crate::dtrace_stop(self.handle) } {
            0 => Ok(()),
            _ => Err(Error::from(self)),
        }
    }

    /// Pauses the DTrace consumer based on the interaction rates with the DTrace framework.
    ///
    /// The `dtrace_sleep()` function calculates the minimum amount of time a consumer needs to pause before it interacts with the DTrace framework again. This is determined based on three rates maintained by DTrace:
    ///
    /// * `switchrate` - Specifies how often the principal buffers must be consumed. Principal buffers are maintained as active and passive pairs per-CPU. The rate at which these buffers switch determines the `switchrate`.
    /// * `statusrate` - Specifies how often the consumer should check the DTrace status.
    /// * `aggrate` - Specifies how often the aggregation buffers are consumed. Aggregation buffers are not maintained as pairs in the same way as principal buffers.
    ///
    /// The function calculates the earliest time for it to wake up based on the last occurrence of these three events and their associated rates. If that earliest time is in the past, the function returns, otherwise it sleeps until that time.
    ///
    /// Note: You do not have to call the `dtrace_sleep()` function itself from a consumer. You can use the `dtrace_getopt()` function to get the values of the appropriate rate and use timers based on those values.
    pub fn dtrace_sleep(&self) {
        unsafe {
            crate::dtrace_sleep(self.handle);
        }
    }

    /// Retrieves the current error number for the DTrace instance.
    ///
    /// # Returns
    ///
    /// Returns the current error number.
    pub fn dtrace_errno(&self) -> c_int {
        unsafe { crate::dtrace_errno(self.handle) }
    }

    /// Retrieves the error message associated with the specified error number.
    ///
    /// # Arguments
    ///
    /// * `handle` - An optional handle to a DTrace instance. If `None`, the error message will be
    ///              retrieved for the global DTrace instance.
    /// * `errno` - The error number.
    ///
    /// # Returns
    ///
    /// Returns the error message as a [`String`].
    pub fn dtrace_errmsg<'a>(handle: Option<&'a Self>, errno: c_int) -> &'a str {
        unsafe {
            let handle = match handle {
                Some(handle) => handle.handle,
                None => std::ptr::null_mut(),
            };
            let msg = crate::dtrace_errmsg(handle, errno);
            let msg = ::core::ffi::CStr::from_ptr(msg);
            msg.to_str().unwrap()
        }
    }

    /// Sets a DTrace option to the specified value.
    ///
    /// # Arguments
    ///
    /// * `option` - The name of the option to set.
    /// * `value` - The value to set for the option.
    ///
    /// For a list of available options, see [DTrace Runtime Options](https://docs.oracle.com/en/operating-systems/oracle-linux/dtrace-v2-guide/dtrace_runtime_options.html).
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the option was set successfully, or an error code if the option could
    /// not be set.
    pub fn dtrace_setopt(&self, option: &str, value: &str) -> Result<(), Error> {
        let option = std::ffi::CString::new(option).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        match unsafe { crate::dtrace_setopt(self.handle, option.as_ptr(), value.as_ptr()) } {
            0 => Ok(()),
            _ => Err(Error::from(self)),
        }
    }

    /// Retrieves the value of the specified DTrace option.
    /// 
    /// # Arguments
    /// 
    /// * `option` - The name of the option to retrieve.
    /// 
    /// # Returns
    /// 
    /// Returns the value of the option if successful, or an error code if the option could not be retrieved.
    pub fn dtrace_getopt(&self, option: &str) -> Result<crate::dtrace_optval_t, Error> {
        let option = std::ffi::CString::new(option).unwrap();
        let mut optval: crate::dtrace_optval_t = 0;
        match unsafe { crate::dtrace_getopt(self.handle, option.as_ptr(), &mut optval) } {
            0 => Ok(optval),
            _ => Err(Error::from(self)),
        }
    }

    /* General Purpose APIs END */

    /* Programming APIs START */
    /// Compiles a DTrace program from a string representation.
    ///
    /// # Arguments
    ///
    /// * `program` - The DTrace program as a string.
    /// * `spec` - spec to indicate the context of the probe you are using.
    ///     * Available values can be found [here](https://docs.oracle.com/en/operating-systems/solaris/oracle-solaris/11.4/dtrace-guide/dtrace_program_strcompile-function.html)
    ///
    /// * `flags` - Flags to control the compilation behavior. Common flags:
    ///     * `DTRACE_C_ZDEFS` - Instructs the compiler to permit probes, whose definitions do not match the existing probes.
    ///                          By default, the compiler does not permit this.
    ///    *  `DTRACE_C_DIFV` - Shows the target language instructions that results from the compilation and additional information to execute the target language instructions.
    ///    *  `DTRACE_C_CPP` - Instructs the compiler to preprocess the input program with the C preprocessor.
    ///
    /// The full list of flags can be found [here](https://github.com/microsoft/DTrace-on-Windows/blob/0adebf25928264dffdc8240e850503865409f334/lib/libdtrace/common/dtrace.h#L115).
    /// * `args` - Optional arguments passed to the program.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a reference to the compiled `dtrace_prog` if successful, or
    /// an error code if the program could not be compiled.
    pub fn dtrace_program_strcompile<'a>(
        &'a self,
        program: &str,
        spec: crate::dtrace_probespec,
        flags: u32,
        args: Option<Vec<String>>,
    ) -> Result<&'a mut crate::dtrace_prog, Error> {
        let program = std::ffi::CString::new(program).unwrap();

        // Break the arguments into argc and argv
        let (argc, argv) = match args {
            None => (0, std::ptr::null()),
            Some(args) => {
                let mut argv: Vec<*mut ::core::ffi::c_char> = Vec::new();
                for arg in args {
                    let arg = std::ffi::CString::new(arg).unwrap();
                    argv.push(arg.as_ptr() as *mut ::core::ffi::c_char);
                }
                (argv.len() as c_int, argv.as_ptr())
            }
        };

        let prog;
        unsafe {
            prog = crate::dtrace_program_strcompile(
                self.handle,
                program.as_ptr(),
                spec,
                flags,
                argc,
                argv,
            );
        }

        if prog.is_null() {
            return Err(Error::from(self));
        }

        unsafe { Ok(&mut *prog) }
    }

    pub fn dtrace_program_fcompile<'a>(
        &'a self,
        file: Option<&utils::File>,
        flags: u32,
        args: Option<Vec<String>>,
    ) -> Result<&'a mut crate::dtrace_prog, Error> {
        // Break the arguments into argc and argv
        let (argc, argv) = match args {
            None => (0, std::ptr::null()),
            Some(args) => {
                let mut argv: Vec<*mut ::core::ffi::c_char> = Vec::new();
                for arg in args {
                    let arg = std::ffi::CString::new(arg).unwrap();
                    argv.push(arg.as_ptr() as *mut ::core::ffi::c_char);
                }
                (argv.len() as c_int, argv.as_ptr())
            }
        };

        let file = match file {
            Some(file) => file.file,
            None => std::ptr::null_mut(),
        };

        let prog;
        unsafe {
            prog = crate::dtrace_program_fcompile(self.handle, file, flags, argc, argv);
        }

        if prog.is_null() {
            return Err(Error::from(self));
        }

        unsafe { Ok(&mut *prog) }
    }

    /// After the D program is compiled, this function is used to create the object file for the program and download the object file to the kernel.
    /// The object file contains all the information necessary for the DTrace framework in the kernel to execute the D program.
    ///
    /// # Arguments
    ///
    /// * `program` - A mutable reference to the data structure representing the compiled program. This is returned by the `dtrace_strcompile()` function.
    /// * `info` - An optional mutable reference to a variable, which contains information about the D program. The definition of the `dtrace_proginfo_t` can be found [`here`](https://github.com/microsoft/DTrace-on-Windows/blob/0adebf25928264dffdc8240e850503865409f334/lib/libdtrace/common/dtrace.h#L106).
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the program execution is successful.
    /// * `Err(errno)` - If the program execution fails. The error number (`errno`) is returned.
    pub fn dtrace_program_exec(
        &self,
        program: &mut crate::dtrace_prog,
        info: Option<&mut crate::dtrace_proginfo>,
    ) -> Result<(), Error> {
        let info = match info {
            Some(info) => info,
            None => std::ptr::null_mut(),
        };
        match unsafe { crate::dtrace_program_exec(self.handle, program, info) } {
            0 => Ok(()),
            _ => Err(Error::from(self)),
        }
    }

    /// Iterates over the statements associated with a D program, calling the specified function on each statement.
    ///
    /// # Arguments
    ///
    /// * `program` -  A mutable reference to the data structure representing the compiled program. This is returned by the `dtrace_strcompile()` function.
    /// * `handler` - The function to call on each statement.
    ///
    ///     The handler function must have the following signature:
    ///     ```rs
    ///     unsafe extern "C" fn( *mut dtrace_hdl_t, *mut dtrace_prog_t, *mut dtrace_stmtdesc_t, *mut c_void) -> c_int
    ///     ```
    /// * `arg` - An optional argument to be passed to the handler function. This argument can maintain any state between successive invocations of the handler function.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the iteration is successful.
    /// * `Err(errno)` - If the iteration fails. The error number (`errno`) is returned.
    pub fn dtrace_stmt_iter(
        &self,
        program: &mut crate::dtrace_prog,
        handler: crate::dtrace_stmt_f,
        arg: Option<*mut ::core::ffi::c_void>,
    ) -> Result<(), Error> {
        let arg = match arg {
            Some(arg) => arg,
            None => std::ptr::null_mut(),
        };

        match unsafe { crate::dtrace_stmt_iter(self.handle, program, handler, arg) } {
            0 => Ok(()),
            _ => Err(Error::from(self)),
        }
    }

    /* Programming APIs END */

    /* Data Consumption APIs START */
    /// Determines the status of the running DTrace instance.
    ///
    /// # Returns
    ///
    /// * `Ok(dtrace_status)` - If the status is successfully determined.
    /// * `Err(errno)` - If the status could not be determined.
    pub fn dtrace_status(&self) -> Result<dtrace_status, Error> {
        match unsafe { crate::dtrace_status(self.handle) } {
            -1 => Err(Error::from(self)),
            status => Ok(dtrace_status::from(status as u32)),
        }
    }

    /// Consumes data from the principal buffers.
    ///
    /// # Arguments
    ///
    /// * `file` - An optional file handle for output.
    /// * `p_hldr` - A pointer to a function that processes an `enabling control block (ECB)`. An `ECB` is a clause from a D program associated with the enabled probe.
    /// * `r_hldr` - A pointer to a function that processes a records from the `ECB`.
    /// * `arg` - An optional argument to be passed to the `p_hldr` and `r_hldr` functions. This argument can maintain any state between successive invocations of the functions.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the consumption is successful.
    /// * `Err(errno)` - If the consumption fails. The error number (`errno`) is returned.
    pub fn dtrace_consume(
        &self,
        file: Option<&utils::File>,
        p_hldr: crate::dtrace_consume_probe_f,
        r_hldr: crate::dtrace_consume_rec_f,
        arg: Option<*mut ::core::ffi::c_void>,
    ) -> Result<(), Error> {
        let file = match file {
            Some(file) => file.file,
            None => std::ptr::null_mut(),
        };
        let arg = match arg {
            Some(arg) => arg,
            None => std::ptr::null_mut(),
        };

        match unsafe { crate::dtrace_consume(self.handle, file, p_hldr, r_hldr, arg) } {
            0 => Ok(()),
            _ => Err(Error::from(self)),
        }
    }

    /// Performs all of the work that must to be done periodically by a DTrace consumer.
    ///
    /// This function corresponds to the `statusrate`, `switchrate`, and `aggrate` rates. It first calls `dtrace_status()` to determine the status of the trace and then calls `dtrace_aggregate_snap()` and `dtrace_consume()` to consume any aggregation buffer or principal buffer data.
    ///
    /// # Arguments
    ///
    /// * `file` - An optional file handle for output.
    /// * `chew` - A function pointer that is called for each enabled probe ID (EPID) that is processed from the buffer.
    /// * `chewrec` - A function pointer that is called for each record that is processed for an EPID.
    /// * `arg` - An optional argument to be passed to the `chew` and `chewrec` functions. This argument can maintain any state between successive invocations of the functions.
    ///
    /// # Returns
    ///
    /// * `DTRACE_WORKSTATUS_OKAY` - If the work is successfully performed.
    /// * `DTRACE_WORKSTATUS_DONE` - If the work is done and no more work is expected.
    /// * `DTRACE_WORKSTATUS_ERROR` - If an error occurs while performing the work.
    pub fn dtrace_work(
        &self,
        file: Option<&utils::File>,
        p_hldr: crate::dtrace_consume_probe_f,
        r_hldr: crate::dtrace_consume_rec_f,
        arg: Option<&mut ::core::ffi::c_void>,
    ) -> Result<crate::dtrace_workstatus_t, Error> {
        let file = match file {
            Some(file) => file.file,
            None => std::ptr::null_mut(),
        };
        let arg = match arg {
            Some(arg) => arg,
            None => std::ptr::null_mut(),
        };
        match unsafe { crate::dtrace_work(self.handle, file, p_hldr, r_hldr, arg) } {
            crate::dtrace_workstatus_t::DTRACE_WORKSTATUS_ERROR => {
                Err(Error::from(self))
            }
            status => Ok(status),
        }
    }

    /* Data Consumption APIs END */

    /* Handler APIs START */
    /// Sets a handler functions for processing trace data.
    /// 
    /// # Arguments
    /// 
    /// * `handler` - An enum variant from [`dtrace_handler`] representing the handler function to be called for each trace record. Possible values:
    ///     * `Buffered(handler)` - The handler function to be called for each buffered trace record.
    ///         * If [`None`] is passed to `dtrace_work`, `dtrace_consume` or `dtrace_aggregate_print` function, then libdtrace makes use of the buffered I/O handler to process buffered trace data.
    ///         * The handler function must have the following signature:
    ///             ```rs
    ///                 unsafe extern "C" fn(*const dtrace_bufdata_t, *mut c_void) -> c_int
    ///             ```
    ///     * `Drop(handler)` - The handler function to be called for each dropped trace record.
    ///         * The handler function must have the following signature:
    ///             ```rs
    ///                 unsafe extern "C" fn(*const dtrace_dropdata_t, *mut c_void) -> c_int
    ///             ```
    ///     * `Err(handler)` - To register a handler function for processing errors such as accessing an invalid address or dividing by zero.
    ///         * The handler function must have the following signature:
    ///             ```rs
    ///                 unsafe extern "C" fn(*const dtrace_errdata_t, *mut c_void) -> c_int
    ///             ```
    ///     * `SetOpt(handler)` - This handler is called whenever a DTrace option is set from inside a D program.
    ///         * The handler function must have the following signature:
    ///             ```rs
    ///                 unsafe extern "C" fn(*const dtrace_setoptdata_t, *mut c_void) -> c_int
    ///             ```
    ///     * `Proc(handler)` - Unsupported.
    /// * `arg` - An optional argument to be passed to the handler function. This argument can maintain any state between successive invocations of the handler function.
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if the handler was set successfully, or an error code if the handler could
    /// not be set.
    pub fn dtrace_register_handler(
        &self,
        handler: crate::types::dtrace_handler,
        arg: Option<*mut ::core::ffi::c_void>,
    ) -> Result<(), Error> {
        let status;
        let arg = match arg {
            Some(arg) => arg,
            None => std::ptr::null_mut(),
        };

        unsafe {
            status = match handler {
                crate::types::dtrace_handler::Buffered(handler) => {
                    crate::dtrace_handle_buffered(self.handle, handler, arg)
                }
                crate::types::dtrace_handler::Drop(handler) => {
                    crate::dtrace_handle_drop(self.handle, handler, arg)
                }
                crate::types::dtrace_handler::Err(handler) => {
                    crate::dtrace_handle_err(self.handle, handler, arg)
                }
                crate::types::dtrace_handler::SetOpt(handler) => {
                    crate::dtrace_handle_setopt(self.handle, handler, arg)
                }
                crate::types::dtrace_handler::Proc(handler) => {
                    crate::dtrace_handle_proc(self.handle, handler, arg)
                }
            };
        }

        if status == 0 {
            Ok(())
        } else {
            Err(Error::from(self))
        }
    }

    /* Handler APIs END */

    /* Aggregation APIs START */
    /// Retrieves aggregation data from the kernel
    ///
    /// This function is called to transfer data from the in-kernel aggregation buffers to the userspace (consumer). The data is not processed at this point.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the aggregation data is successfully retrieved.
    /// * `Err(errno)` - If the aggregation data could not be retrieved. The error number (`errno`) is returned.
    pub fn dtrace_aggregate_snap(&self) -> Result<(), Error> {
        match unsafe { crate::dtrace_aggregate_snap(self.handle) } {
            0 => Ok(()),
            _ => Err(Error::from(self)),
        }
    }

    /// Processes DTrace aggregate data.
    ///
    /// The function can be passed a specific `walk()` function. If passed `None`, it defaults to the `dtrace_aggregate_walk_sorted()` function,
    /// and the callback function passed to the `walk()` function is the default function that the libdtrace library uses to print aggregate data.
    ///
    /// # Arguments
    ///
    /// * `file` - An optional file handle for output.
    /// * `handler` - A function pointer that is called for each aggregate buffer that is processed.
    /// * `arg` - An optional argument to be passed to the `handler` function. This argument can maintain any state between successive invocations of the function.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the processing is successful.
    /// * `Err(i32)` - If the processing fails. The error number is returned.
    pub fn dtrace_aggregate_print(
        &self,
        file: Option<&utils::File>,
        handler: crate::dtrace_aggregate_walk_f,
    ) -> Result<(), Error> {
        let file = match file {
            Some(file) => file.file,
            None => std::ptr::null_mut(),
        };

        match unsafe { crate::dtrace_aggregate_print(self.handle, file, handler) }
        {
            0 => Ok(()),
            _ => Err(Error::from(self)),
        }
    }

    /// Processes DTrace aggregate data.
    ///
    /// # Arguments
    ///
    /// * `handler` - A function pointer that is called for each aggregate buffer that is processed.
    /// * `arg` - An optional argument to be passed to the `handler` function. This argument can maintain any state between successive invocations of the function.
    /// * `order` - The order in which the data is processed. One of the members of the [`dtrace_aggwalk_order`] enum.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the processing is successful.
    /// * `Err(i32)` - If the processing fails. The error number is returned.
    pub fn dtrace_aggregate_walk(
        &self,
        handler: crate::dtrace_aggregate_f,
        arg: Option<*mut ::core::ffi::c_void>,
        order: dtrace_aggwalk_order,
    ) -> Result<(), Error> {
        let status;
        let arg = match arg {
            Some(arg) => arg,
            None => std::ptr::null_mut(),
        };
        unsafe {
            status = match order {
                dtrace_aggwalk_order::None => {
                    crate::dtrace_aggregate_walk(self.handle, handler, arg)
                }
                dtrace_aggwalk_order::Sorted | dtrace_aggwalk_order::ValSorted => {
                    crate::dtrace_aggregate_walk_sorted(self.handle, handler, arg)
                }
                dtrace_aggwalk_order::KeySorted => {
                    crate::dtrace_aggregate_walk_keysorted(self.handle, handler, arg)
                }
                dtrace_aggwalk_order::KeyVarSorted => {
                    crate::dtrace_aggregate_walk_keyvarsorted(self.handle, handler, arg)
                }
                dtrace_aggwalk_order::ValVarSorted => {
                    crate::dtrace_aggregate_walk_valvarsorted(self.handle, handler, arg)
                }
                dtrace_aggwalk_order::KeyRevSorted => {
                    crate::dtrace_aggregate_walk_keyrevsorted(self.handle, handler, arg)
                }
                dtrace_aggwalk_order::ValRevSorted => {
                    crate::dtrace_aggregate_walk_valrevsorted(self.handle, handler, arg)
                }
                dtrace_aggwalk_order::KeyVarRevSorted => {
                    crate::dtrace_aggregate_walk_keyvarrevsorted(self.handle, handler, arg)
                }
                dtrace_aggwalk_order::ValVarRevSorted => {
                    crate::dtrace_aggregate_walk_valvarrevsorted(self.handle, handler, arg)
                }
            };
        }

        if status == 0 {
            Ok(())
        } else {
            Err(Error::from(self))
        }
    }

    /* Aggregation APIs END */
}
