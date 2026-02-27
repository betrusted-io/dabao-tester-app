mod dabao_tester;

use bao1x_api::IoSetup;
use bao1x_api::IoxEnable;
use bao1x_api::IoxPort;
use bao1x_api::bio::*;
use bao1x_api::bio_resources::*;
use bao1x_hal::bio::{Bio, CoreCsr};
use dabao_tester::*;
use utralib::utra::bio_bdma;

pub struct DabaoTester {
    bio_ss: Bio,
    // handles have to be kept around or else the underlying CSR is dropped
    _rx_handle: CoreHandle,
    // the CoreCsr is a convenience object that manages the CSR view of the handle
    rx: CoreCsr,
    // tracks the resources used by the object
    resource_grant: ResourceGrant,
}

impl Resources for DabaoTester {
    fn resource_spec() -> ResourceSpec {
        ResourceSpec {
            claimer: "colorwheel".to_string(),
            cores: vec![CoreRequirement::Any],
            fifos: vec![Fifo::Fifo0],
            static_pins: vec![28, 27, 26, 25, 24, 23, 19, 18, 17, 16, 14, 13, 12, 11, 1, 2, 3, 4, 5],
            dynamic_pin_count: 0,
        }
    }
}

impl Drop for DabaoTester {
    fn drop(&mut self) {
        for &core in self.resource_grant.cores.iter() {
            self.bio_ss.de_init_core(core).unwrap();
        }
        self.bio_ss.release_resources(self.resource_grant.grant_id).unwrap();
    }
}

impl DabaoTester {
    pub fn new(io_mode: Option<IoConfigMode>) -> Result<Self, BioError> {
        let mut bio_ss = Bio::new();
        // claim core resource and initialize it
        let resource_grant = bio_ss.claim_resources(&Self::resource_spec())?;
        let config = CoreConfig { clock_mode: bao1x_api::bio::ClockMode::TargetFreqInt(1_000_000) };
        bio_ss.init_core(resource_grant.cores[0], dabao_tester_bio_code(), config)?;
        bio_ss.set_core_run_state(&resource_grant, true);

        // configure pullups/schmitt triggers on everything
        let iox = bao1x_api::iox::IoxHal::new();
        for pin in DabaoTester::resource_spec().static_pins {
            iox.setup_pin(
                if pin <= 15 { IoxPort::PB } else { IoxPort::PC },
                if pin <= 15 { pin } else { pin - 16 },
                None,
                None,
                Some(IoxEnable::Enable),
                Some(IoxEnable::Enable),
                None,
                None,
            );
        }

        // now configure the claimed resource
        let mut io_config = IoConfig::default();

        io_config.mode = io_mode.unwrap_or(IoConfigMode::Overwrite);
        bio_ss.setup_io_config(io_config).unwrap();

        // safety: fifo is stored in this object so they aren't Drop'd before the object is
        // destroyed
        let rx_handle = unsafe { bio_ss.get_core_handle(Fifo::Fifo0) }?.expect("Didn't get FIFO0 handle");

        Ok(Self { bio_ss, rx: CoreCsr::from_handle(&rx_handle), _rx_handle: rx_handle, resource_grant })
    }

    /// Call this immediately after setting up Colorwheel, because when the object goes out
    /// of scope, the program stops running. This basically is just a placeholder to keep
    /// the object around long enough.
    pub fn changes(&mut self) -> Vec<u32> {
        let mut changes = Vec::new();
        while self.rx.csr.rf(bio_bdma::SFR_FLEVEL_PCLK_REGFIFO_LEVEL0) != 0 {
            changes.push(self.rx.csr.r(bio_bdma::SFR_RXF0));
        }
        return changes;
    }
}
