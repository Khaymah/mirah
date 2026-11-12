#![allow(
    unsafe_code,
    reason = "job objects have no safe wrapper; the handle is owned and closed exactly once"
)]

use std::os::windows::io::RawHandle;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject,
};

const KILLED: u32 = 1;

pub(crate) struct Job(HANDLE);

unsafe impl Send for Job {}

impl Job {
    pub(crate) fn contain(process: RawHandle) -> Option<Self> {
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return None;
            }
            let job = Self(job);
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let size =
                u32::try_from(std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>()).ok()?;
            let limited = SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                size,
            ) != 0;
            let assigned = AssignProcessToJobObject(job.0, process.cast()) != 0;
            (limited && assigned).then_some(job)
        }
    }

    pub(crate) fn end(&mut self) {
        unsafe {
            TerminateJobObject(self.0, KILLED);
        }
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::io::AsRawHandle;
    use std::time::{Duration, Instant};

    #[test]
    fn ending_the_job_ends_the_program_and_what_it_started() {
        let mut child = std::process::Command::new("cmd")
            .args(["/d", "/c", "ping -n 60 127.0.0.1 >nul"])
            .spawn()
            .expect("start cmd");
        let mut job = Job::contain(child.as_raw_handle()).expect("job");
        job.end();
        let started = Instant::now();
        let mut ended = false;
        while started.elapsed() < Duration::from_secs(10) {
            if child.try_wait().ok().flatten().is_some() {
                ended = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if !ended {
            let _ = child.kill();
        }
        assert!(ended, "the program outlived its job");
    }
}
