mod errors;

use std::{thread, time::Duration};

use embedded_hal as hal;
use errors::Error;
use heapless::Vec as StackVec;
use nb::block;

const SIM800_NUMBER_LEN: usize = 40;
const SIM800_RCV_BUF_LEN: usize = 1600;
//
use embedded_hal::blocking::delay::DelayUs;

pub struct Sim800Module<UART, DELAY> {
    uart: UART,
    rcv_buf: StackVec<u8, SIM800_RCV_BUF_LEN>,
    delay: DELAY,
}

impl<UART, DELAY, E> Sim800Module<UART, DELAY>
where
    UART: hal::serial::Read<u8, Error = E> + hal::serial::Write<u8, Error = E>,
    E: std::fmt::Debug,
{
    pub fn new(uart: UART, delay: DELAY) -> Self {
        Sim800Module {
            uart,
            rcv_buf: StackVec::new(),
            delay,
        }
    }

    pub fn send_at_cmd_wait_resp(
        &mut self,
        at_cmd: &[u8], //
        toc: u16,      // timeout for first char
        to: u16,
    ) -> Result<(), Error> {
        loop {
            let res = self.uart.read();
            match res {
                Err(nb::Error::Other(_)) => {
                    //write_log(b"SIM e0");
                    break;
                }
                Err(nb::Error::WouldBlock) => {
                    // Buffer is empty
                    break;
                }
                Ok(_) => {}
            }
        }
        for cmd in at_cmd {
            block!(self.uart.write(*cmd)).ok();
        }
        self.rcv_buf.clear();
        //ctx.at_timer.reset(); // reset timeout counter
        let mut got_first_char = false;
        let mut w1_cycles = 0;
        let mut w2_cycles = 0;
        loop {
            let res = self.uart.read();

            match res {
                Err(nb::Error::Other(_)) => {
                    return Err(Error::SerialError);
                }
                Err(nb::Error::WouldBlock) => {
                    // TODO(jj): replace with no_std enabled delay
                    thread::sleep(Duration::from_millis(100));
                    break;
                }
                Ok(x) => {
                    //print!("{}", char::from(x).to_string());
                    if self.rcv_buf.len() < SIM800_RCV_BUF_LEN {
                        self.rcv_buf.push(x).unwrap();
                    } else {
                        break;
                    }
                    got_first_char = true;
                    w2_cycles = 0;
                    continue;
                }
                Err(err) => {
                    todo!();
                }
            }
        }
        if self.rcv_buf.len() == 0 {
            return Err(Error::SerialNoData);
        }
        Ok(())
    }

    fn send_at_cmd_wait_resp_n(
        &mut self,
        at_cmd: &[u8],
        ans: &[u8],
        toc: u16, // timeout for first char
        to: u16,  // timeout after last char
        tries: u8,
    ) -> Result<(), Error> {
        // no of attempts
        // checks if reply from SIM800 contains and using tries attempts
        let mut reply: bool = false;
        for _ in 0..tries {
            match self.send_at_cmd_wait_resp(at_cmd, toc, to) {
                Ok(_) => {}
                Err(Error::SerialNoData) => continue,
                Err(val) => return Err(val),
            };
            if Self::buf_contains(&self.rcv_buf, ans) {
                reply = true;
                break;
            }
            //ctx.delay.delay_ms(500_u16); // delay between attempts
            //ctx.watchdog.feed();
        }
        if reply {
            return Ok(());
        }
        Err(Error::CmdFail)
    }

    fn init_set_0(&mut self) -> Result<(), Error> {
        //match &self.state_machine {
        //    SimStateWrapper::Initial800(_) => return Ok(()),
        //    _ => {}
        //}
        // Reset to the factory settings
        self.send_at_cmd_wait_resp_n(b"AT&F\n", b"OK\r", 100, 10, 3)?;
        // switch off echo
        self.send_at_cmd_wait_resp_n(b"ATE0\n", b"OK\r", 50, 10, 3)?;
        // setup fixed baud rate 9600
        self.send_at_cmd_wait_resp_n(b"AT+IPR=9600\n", b"OK\r", 100, 10, 3)
    }

    fn gsm_busy(&mut self, set_to: bool) -> Result<(), Error> {
        self.send_at_cmd_wait_resp_n(
            if set_to {
                b"AT+GSMBUSY=1\n"
            } else {
                b"AT+GSMBUSY=0\n"
            },
            b"OK\r",
            100,
            10,
            3,
        )
    }

    fn sim800_test(&mut self) -> Result<(), Error> {
        self.send_at_cmd_wait_resp_n(b"ATI\n", b"OK\r", 100, 10, 3)
    }

    pub fn buf_contains(buffer: &[u8], pattern: &[u8]) -> bool {
        let psize = pattern.len();
        let bsize = buffer.len();
        for i in 0..bsize {
            let rlimit = i + psize;
            if rlimit > bsize {
                break;
            }
            let sl = &buffer[i..rlimit];
            if sl == pattern {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use linux_embedded_hal::Serial;

    #[test]
    fn it_works() {
        let serial = Serial::open(Path::new(&"/dev/ttyUSB0")).expect("Creating TTYPort failed");
        let mut sim = Sim800Module::new(serial, ());
        sim.init_set_0().unwrap();
        sim.gsm_busy(true); //  disable all incoming calls
        sim.sim800_test();
    }
}
