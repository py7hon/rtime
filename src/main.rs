use std::env;
use std::ffi::c_void;
use std::io::ErrorKind;
use std::os::windows::io::AsRawHandle;
use std::process::{exit, Command};
use std::ptr;
use std::time::Instant;

use windows_sys::Win32::Foundation::{CloseHandle, FILETIME};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicAccountingInformation,
    QueryInformationJobObject, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
};
use windows_sys::Win32::System::Threading::GetProcessTimes;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("usage: rtime <command> [argument...]");
        exit(1);
    }

    let cmd = &args[0];
    let cmd_args = &args[1..];

    let start_time = Instant::now();

    unsafe {
        let job_handle = CreateJobObjectW(ptr::null(), ptr::null());
        if job_handle == 0 {
            eprintln!("rtime: failed create Windows Job Object");
            exit(1);
        }

        // Auto-Fallback System
        let mut child = match Command::new(cmd).args(cmd_args).spawn() {
            Ok(c) => c,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    Command::new("cmd")
                        .arg("/C")
                        .arg(cmd)
                        .args(cmd_args)
                        .spawn()
                        .unwrap_or_else(|err| {
                            eprintln!("rtime: failed execute '{}' via cmd: {}", cmd, err);
                            CloseHandle(job_handle);
                            exit(127);
                        })
                } else {
                    eprintln!("rtime: failed execute '{}': {}", cmd, e);
                    CloseHandle(job_handle);
                    exit(127);
                }
            }
        };

        let process_handle = child.as_raw_handle() as _;
        
        AssignProcessToJobObject(job_handle, process_handle);

        let status = child.wait().unwrap_or_else(|err| {
            eprintln!("rtime: error while wait process: {}", err);
            CloseHandle(job_handle);
            exit(1);
        });

        let real_duration = start_time.elapsed();
        
        let mut job_info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = std::mem::zeroed();
        let query_result = QueryInformationJobObject(
            job_handle,
            JobObjectBasicAccountingInformation,
            &mut job_info as *mut _ as *mut c_void,
            std::mem::size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
            ptr::null_mut(),
        );

        eprintln!();

        if query_result != 0 {
            let mut user_secs = job_info.TotalUserTime as f64 / 10_000_000.0;
            let mut sys_secs = job_info.TotalKernelTime as f64 / 10_000_000.0;

            // DUAL-ENGINE FALLBACK
            if user_secs == 0.0 && sys_secs == 0.0 {
                let mut c_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
                let mut e_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
                let mut k_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
                let mut u_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };

                if GetProcessTimes(process_handle, &mut c_time, &mut e_time, &mut k_time, &mut u_time) != 0 {
                    let k_total = ((k_time.dwHighDateTime as u64) << 32) | k_time.dwLowDateTime as u64;
                    let u_total = ((u_time.dwHighDateTime as u64) << 32) | u_time.dwLowDateTime as u64;
                    
                    sys_secs = k_total as f64 / 10_000_000.0;
                    user_secs = u_total as f64 / 10_000_000.0;
                }
            }

            eprintln!("real\t{:.3}s", real_duration.as_secs_f64());
            eprintln!("user\t{:.3}s", user_secs);
            eprintln!("sys\t{:.3}s", sys_secs);
        } else {
            eprintln!("real\t{:.3}s", real_duration.as_secs_f64());
            eprintln!("rtime: failed read CPU from Windows");
        }

        CloseHandle(job_handle);
        exit(status.code().unwrap_or(1));
    }
}