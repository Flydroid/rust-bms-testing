#![deny(warnings)]
#![no_main]
#![no_std]


mod sys; // imports the sys module
mod panic; //imports the panic handler
mod thermistor;

#[macro_use(block)]
extern crate nb;

#[allow(unused)]
use cortex_m_rt::entry;
//use nb::block
/* Select which parts to use from the stm327xx_hal */
use stm32h7xx_hal::{pac, prelude::*, rcc,gpio::{Alternate, Pin}, time::Hertz,spi,time::MilliSeconds}; 
/*Setup RTT Logging */
pub use rtt_target::{rprintln as log, rtt_init_print as log_init};
use ltc681x::config::Configuration;

use ltc681x::ltc6811::{CellSelection,GPIOSelection};
use ltc681x::monitor::{ADCMode, LTC681X, LTC681XClient, PollClient};

/* Define processor frequency */
pub const CORE_FREQUENCY: Hertz = Hertz::from_raw(480_000_000);

use thermistor::Thermistor;


#[entry]
fn main() -> ! {
    log_init!(); // Init rtt logging

    sys::Clk::new().reset().enable_ext_clock();

    
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    log!("Setup Power...");
    // Constrain and Freeze power
    let pwrcfg: stm32h7xx_hal::pwr::PowerConfiguration = dp.PWR.constrain().vos0(&dp.SYSCFG).freeze();
    log!("Setup RCC...");
    // Constrain and Freeze clock
    let ccdr = dp
        .RCC
        .constrain()
        .use_hse(25.MHz())
        .bypass_hse()
        .sys_ck(CORE_FREQUENCY)
        .hclk(240.MHz())
        .pll1_strategy(rcc::PllConfigStrategy::Iterative)
        .freeze(pwrcfg, &dp.SYSCFG);

    /*Get GPIO K peripheral */
    let gpio_k = dp.GPIOK.split(ccdr.peripheral.GPIOK);
    /*Get GPIO J peripheral */
    let gpio_j = dp.GPIOJ.split(ccdr.peripheral.GPIOJ);
    /*Get GPIO J peripheral */
    let gpio_h = dp.GPIOH.split(ccdr.peripheral.GPIOH);
    /*Get GPIO D peripheral */
    let gpio_d = dp.GPIOD.split(ccdr.peripheral.GPIOD);
    /*Get GPIO C peripheral */
    let gpio_c = dp.GPIOC.split(ccdr.peripheral.GPIOC);
     /*Get GPIO E peripheral */
     let gpio_e = dp.GPIOE.split(ccdr.peripheral.GPIOE);

    let mosi: Pin<'J', 10, Alternate<5>> = gpio_j.pj10.into_alternate();
    let miso: Pin<'J', 11, Alternate<5>> = gpio_j.pj11.into_alternate();
    let sck:  Pin<'H', 6, Alternate<5>>  = gpio_h.ph6.into_alternate();

    let mut cs_pin = gpio_k.pk1.into_push_pull_output();
    cs_pin.set_high();
    let mut isospi_master1 = gpio_d.pd5.into_push_pull_output();
    isospi_master1.set_high();
    let mut isospi_master2 = gpio_e.pe3.into_push_pull_output();
    isospi_master2.set_high();


    let mut status_pin = gpio_c.pc7.into_push_pull_output();

    // Get the delay provider.
    let delay = cp.SYST.delay(ccdr.clocks);

    // Configure the timer.
    let mut timer: stm32h7xx_hal::timer::Timer<pac::TIM2> = dp.TIM2.timer(1.Hz(), ccdr.peripheral.TIM2, &ccdr.clocks);

   // Initialise the SPI peripheral.
    let spi_bus: spi::Spi<pac::SPI5, spi::Enabled> = dp.SPI5.spi(
        (sck,miso,mosi),
        spi::MODE_0,
        1u32.MHz(),
        ccdr.peripheral.SPI5,
        &ccdr.clocks,

    );

    
    isospi_master2.set_high();

    
    
    log!("Setup Done");


    // LTC6811 device
    let mut client: LTC681X<spi::Spi<pac::SPI5, spi::Enabled>, Pin<'K', 1, stm32h7xx_hal::gpio::Output>, ltc681x::monitor::PLADCPolling, ltc681x::ltc6811::LTC6811, 2, stm32h7xx_hal::delay::Delay> = LTC681X::ltc6811(spi_bus, cs_pin,delay)
        .enable_pladc_polling();

    let mut config: Configuration = Configuration::default();





        // Set over-voltage limit to 4.25 V
    config.set_ov_comp_voltage(4_250_000).unwrap();

    // Sets under-voltage limit to 3.0 V
    config.set_uv_comp_voltage(3_000_000).unwrap();
    config.enable_reference_power();
    config.disable_discharge_timer();
    let config2 = config.clone();

    let _ = client.wake_up();

    client.write_configuration([config,config2]).unwrap();

    let _ = client.wake_up();


    let rega = client.read_register(ltc681x::ltc6811::Register::ConfigurationA);
    match rega{
        Ok(array) =>{
            log!("RegA : 0x{:X}",array[0][0]);
            log!("RegA0 : 0x{:X}",array[0][0].to_be_bytes()[1]);
            log!("RegA1 : 0x{:X}",array[0][0].to_be_bytes()[0]);
            log!("RegA2 : 0x{:X}",array[0][1].to_be_bytes()[1]);
            log!("RegA3 : 0x{:X}",array[0][2].to_be_bytes()[0]);
            log!("RegA4 : 0x{:X}",array[0][2].to_be_bytes()[1]);
            log!("RegA5 : 0x{:X}",array[0][2].to_be_bytes()[0]);

        }
        Err(err) =>{
            log!("Error Read ConfigA: {:?}",err);
        }
    }



    // Configure PK5 as output.
    let _led = gpio_k.pk5.into_push_pull_output();

    let ntc_d: Thermistor = Thermistor::new(
        //25_f32,            // 25°C
        3_f32,      // 3V
        10_f32,        // 10k resistor for voltage bridge
        //3380_u16,       // Thermistor constant
        -20_f32,    // lower temp limit
        80_f32,   // upper temp limit
        1_f32,  // temperature increment in LUT
        101_u16,    // length of LUT
        thermistor::LUT_NTU_CELL_MODULE_D // pointer to lut
    );
    



    loop {

        let _ = client.wake_up();
        // Starts conversion for cell group 1
        let _rst = client.start_conv_cells(ADCMode::Normal, CellSelection::All, true);
        log!("Poll ADC Ready");
        let _ = client.adc_ready();
        let _ = client.wake_up();
        // Returns the value of cell group A. In case of LTC613: cell 1, 7 and 13
        let volts = client.read_voltages(CellSelection::All).unwrap();
        for c in 0..2{
            for i in 0..12{
                log!("ic{} cell{} voltage: {:?}",c,i,volts[c][i].voltage);
            }
        }
        log!("Wakeup");
        let _ = client.wake_up();
        let _ = client.start_conv_gpio(ADCMode::Normal, GPIOSelection::All);
           
        let reg_aux_a = client.read_register(ltc681x::ltc6811::Register::AuxiliaryA).unwrap();
        let reg_aux_b = client.read_register(ltc681x::ltc6811::Register::AuxiliaryB).unwrap();
        let mut gpio_voltage: [[u16;6];2] = [[0;6];2];
        for ic in 0..2{
            let mut ccat: [u16;6] = [0;6];
            ccat[..3].copy_from_slice(&reg_aux_a[ic]);
            ccat[3..].copy_from_slice(&reg_aux_b[ic]);
            gpio_voltage[ic] = ccat;
        }

        for c in 0..2{
            for i in 0..5{
                log!("ic{} gpio{} temps: {:?}",c,i,ntc_d.convert_volt_to_temp(gpio_voltage[c][i]));
            }
        }
        // log!("Read Registers");
        // let rega = client.read_register(ltc681x::ltc6811::Register::ConfigurationA);
        // match rega{
        //     Ok(array) =>{
        //         log!("RegA0 : 0x{:X}",array[0][0].to_be_bytes()[1]);
        //         log!("RegA1 : 0x{:X}",array[0][0].to_be_bytes()[0]);
        //         log!("RegA2 : 0x{:X}",array[0][1].to_be_bytes()[1]);
        //         log!("RegA3 : 0x{:X}",array[0][2].to_be_bytes()[0]);
        //         log!("RegA4 : 0x{:X}",array[0][2].to_be_bytes()[1]);
        //         log!("RegA5 : 0x{:X}",array[0][2].to_be_bytes()[0]);

        //     }
        //     Err(err) =>{
        //         log!("Error Read ConfigA: {:?}",err);
        //     }
        // }


        status_pin.toggle();
        timer.start(MilliSeconds::from_ticks(50).into_rate());
        block!(timer.wait()).ok();
    }

}
