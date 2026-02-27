use String;

use crate::{CommonEnv, ShellCmdApi, c::dabao_tester::DabaoTester};

pub struct Test {
    test_core: DabaoTester,
}
impl Test {
    pub fn new() -> Self { Self { test_core: DabaoTester::new(None).unwrap() } }
}

impl<'a> ShellCmdApi<'a> for Test {
    cmd_api!(test);

    // inserts boilerplate for command API

    fn process(&mut self, args: String, _env: &mut CommonEnv) -> Result<Option<String>, xous::Error> {
        use core::fmt::Write;
        let mut ret = String::new();

        #[allow(unused_variables)]
        let helpstring = "test [proc] [freemem] [interrupts] [panic] [timer] [env]";

        let mut tokens = args.split(' ');

        if let Some(sub_cmd) = tokens.next() {
            match sub_cmd {
                "proc" => {
                    // hard coded - debug feature - if the platform ABI changes its name or opcode map this
                    // can break, but also this routine is not meant for public
                    // consumption and coding it here avoids breaking dependencies to the Xous API crate.
                    let page_buf = xous::PageBuf::new();
                    xous::rsyscall(xous::SysCall::PlatformSpecific(2, page_buf.as_ptr(), 0, 0, 0, 0, 0))
                        .unwrap();

                    log::info!("Process listing:");
                    for line in page_buf.as_str().lines() {
                        log::info!("{}", line);
                    }
                }
                "freemem" => {
                    // hard coded - debug feature - if the platform ABI changes its name or opcode map this
                    // can break, but also this routine is not meant for public
                    // consumption and coding it here avoids breaking dependencies to the Xous API crate.
                    let page_buf = xous::PageBuf::new();
                    xous::rsyscall(xous::SysCall::PlatformSpecific(1, page_buf.as_ptr(), 0, 0, 0, 0, 0))
                        .unwrap();

                    log::info!("RAM usage:");
                    for line in page_buf.as_str().lines() {
                        log::info!("{}", line);
                    }
                }
                "interrupts" => {
                    // hard coded - debug feature - if the platform ABI changes its name or opcode map this
                    // can break, but also this routine is not meant for public
                    // consumption and coding it here avoids breaking dependencies to the Xous API crate.
                    let page_buf = xous::PageBuf::new();
                    xous::rsyscall(xous::SysCall::PlatformSpecific(3, page_buf.as_ptr(), 0, 0, 0, 0, 0))
                        .unwrap();

                    log::info!("Interrupt handlers:");
                    for line in page_buf.as_str().lines() {
                        log::info!("{}", line);
                    }
                }
                "env" => {
                    log::info!("{:?}", std::env::vars());
                }
                "dabao" => {
                    // drain any stale changes
                    let _ = self.test_core.changes();
                    log::info!("TEST.START");

                    let mut expected_pins: Vec<u32> =
                        vec![28, 27, 26, 25, 24, 23, 19, 18, 17, 16, 14, 13, 12, 11, 1, 2, 3, 4, 5];
                    let start = std::time::Instant::now();
                    while std::time::Instant::now().duration_since(start).as_secs() < 5 {
                        let pins = self.test_core.changes();
                        for pin_mask in &pins {
                            for bit in 0..32u32 {
                                if pin_mask & (1 << bit) != 0 {
                                    expected_pins.retain(|&p| p != bit);
                                    log::info!("saw pin {}", bit);
                                }
                            }
                        }
                        if expected_pins.is_empty() {
                            break;
                        }
                    }
                    if expected_pins.is_empty() {
                        log::info!("TEST.PASSING");
                    } else {
                        log::info!("TEST.FAIL");
                    }
                }
                _ => {
                    write!(ret, "{}", helpstring).unwrap();
                }
            }
        } else {
            write!(ret, "{}", helpstring).unwrap();
        }
        Ok(Some(ret))
    }
}
