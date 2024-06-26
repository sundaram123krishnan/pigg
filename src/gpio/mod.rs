use std::fmt;
use std::fs::File;
use std::io;
use std::io::{BufReader, Write};

use serde::{Deserialize, Serialize};

pub type BCMPinNumber = u8;
pub type BoardPinNumber = u8;

/// An input can be configured to have an optional pull-up or pull-down
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum InputPull {
    PullUp,
    PullDown,
    None,
}

impl fmt::Display for InputPull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputPull::PullUp => write!(f, "Pull Up"),
            InputPull::PullDown => write!(f, "Pull Down"),
            InputPull::None => write!(f, "None"),
        }
    }
}

/// For SPI interfaces see [here](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
///
/// Standard mode
/// In Standard SPI mode the peripheral implements the standard three-wire serial protocol
/// * SCLK - serial clock
/// * CE   - chip enable (often called chip select)
/// * MOSI - master out slave in
/// * MISO - master in slave out
///
/// Bidirectional mode
/// In bidirectional SPI mode the same SPI standard is implemented, except that a single wire
/// is used for data (MOMI) instead of the two used in standard mode (MISO and MOSI).
/// In this mode, the MOSI pin serves as MOMI pin.
/// * SCLK - serial clock
/// * CE   - chip enable (often called chip select)
/// * MOMI - master out master in
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum PinFunction {
    None,

    /// Power and Ground functions
    Power3V3,
    Power5V,
    Ground,

    /// GPIO functions
    Input(Option<InputPull>),
    Output(Option<PinLevel>),

    /// General Purpose Clock functions (from https://pinout.xyz/pinout/gpclk)
    GPCLK0,
    GPCLK1,
    GPCLK2,

    /// I2C bus functions
    I2C1_SDA,
    I2C1_SCL,
    I2C3_SDA,
    I2C3_SCL,
    I2C4_SDA,
    I2C4_SCL,
    I2C5_SDA,
    I2C5_SCL,
    I2C6_SDA,
    I2C6_SCL,

    /// SPI Interface #0
    SPI0_MOSI,
    /// Bi-directional mode
    SPI0_MOMI,
    SPI0_MISO,
    SPI0_SCLK,
    SPI0_CE0_N,
    SPI0_CE1_N,

    // SPI Interface #0
    SPI1_MOSI,
    /// Bi-directional mode
    SPI1_MOMI,
    SPI1_MISO,
    SPI1_SCLK,
    SPI1_CE0_N,
    SPI1_CE1_N,
    SPI1_CE2_N,

    /// PWM functions - two pins each use these
    PWM0,
    PWM1,

    /// UART functions
    /// UART0 - Transmit
    UART0_TXD,
    /// UART0 - Receive
    UART0_RXD,

    /// PCM functions - how uncompressed digital audio is encoded
    PCM_FS,
    /// PCM Data In
    PCM_DIN,
    /// PCM Data Out
    PCM_DOUT,
    /// PCM CLock
    PCM_CLK,

    /// HAT ID related functions - two pins to talk to HAT EEPROM via I2C
    I2C_EEPROM_ID_SD,
    I2C_EEPROM_ID_SC,
}

impl fmt::Display for PinFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Remove anything after the first '(' of debug output
        let full = format!("{:?}", self);
        write!(f, "{}", full.split_once('(').unwrap_or((&full, "")).0)
    }
}

// [board_pin_number] refer to the pins by the number of the pin printed on the board
// [bcm_pin_number] refer to the pins by the "Broadcom SOC channel" number,
// these are the numbers after "GPIO"
#[derive(Debug, Clone)]
pub struct PinDescription {
    pub board_pin_number: BoardPinNumber,
    pub bcm_pin_number: Option<BCMPinNumber>,
    pub name: &'static str,
    pub options: &'static [PinFunction], // The set of functions the pin can have, chosen by user config
}

impl fmt::Display for PinDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Board Pin #: {}", self.board_pin_number)?;
        writeln!(f, "\tBCM Pin #: {:?}", self.bcm_pin_number)?;
        writeln!(f, "\tName Pin #: {}", self.name)?;
        writeln!(f, "\tFunctions #: {:?}", self.options)
    }
}

/// A vector of tuples of (bcm_pin_number, PinFunction)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GPIOConfig {
    pub configured_pins: Vec<(BCMPinNumber, PinFunction)>,
}

impl fmt::Display for GPIOConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.configured_pins.is_empty() {
            writeln!(f, "No Pins are Configured")
        } else {
            writeln!(f, "Configured Pins:")?;
            for (bcm_pin_number, pin_function) in &self.configured_pins {
                writeln!(f, "\tBCM Pin #: {bcm_pin_number} - {}", pin_function)?;
            }
            Ok(())
        }
    }
}

impl GPIOConfig {
    /// Load a new GPIOConfig from the file named `filename`
    #[cfg(feature = "gui")]
    #[cfg_attr(feature = "gui", allow(dead_code))]
    // TODO take AsPath/AsRef etc
    pub fn load(filename: &str) -> io::Result<GPIOConfig> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    /// Save this GPIOConfig to the file named `filename`
    #[cfg(feature = "gui")]
    #[cfg_attr(feature = "gui", allow(dead_code))]
    pub fn save(&self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;
        let contents = serde_json::to_string(self)?;
        file.write_all(contents.as_bytes())
    }
}

pub type PinLevel = bool;

#[cfg(test)]
mod test {
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use tempfile::tempdir;

    use crate::gpio::{GPIOConfig, PinFunction};
    use crate::gpio::InputPull::PullUp;

    #[test]
    fn create_a_config() {
        let config = GPIOConfig::default();
        assert!(config.configured_pins.is_empty());
    }

    #[test]
    fn save_one_pin_config_input_no_pullup() {
        let config = GPIOConfig {
            configured_pins: vec![(1, PinFunction::Input(None))],
        };

        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.piggui");

        config.save(test_file.to_str().unwrap()).unwrap();

        let pin_config = r#"{"configured_pins":[[1,{"Input":null}]]}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
    }

    #[test]
    fn load_one_pin_config_input_no_pull() {
        let pin_config = r#"{"configured_pins":[[1,{"Input":null}]]}"#;
        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.piggui");
        let mut file = File::create(&test_file).expect("Could not create test file");
        file.write_all(pin_config.as_bytes())
            .expect("Could not write to test file");
        let config = GPIOConfig::load(test_file.to_str().unwrap()).unwrap();
        assert_eq!(config.configured_pins.len(), 1);
        assert_eq!(config.configured_pins[0].0, 1);
        assert_eq!(config.configured_pins[0].1, PinFunction::Input(None));
    }

    #[test]
    fn load_test_file() {
        let root = std::env::var("CARGO_MANIFEST_DIR").expect("Could not get manifest dir");
        let mut path = PathBuf::from(root);
        path = path.join("configs/andrews_board.piggui");
        let config = GPIOConfig::load(path.to_str().expect("Could not get Path as str"))
            .expect("Could not load GPIOConfig from path");
        assert_eq!(config.configured_pins.len(), 2);
        // GPIO17 configured as an Output - set to true (high) level
        assert_eq!(config.configured_pins[0].0, 17);
        assert_eq!(config.configured_pins[0].1, PinFunction::Output(Some(true)));

        // GPIO26 configured as an Input - with an internal PullUp
        assert_eq!(config.configured_pins[1].0, 26);
        assert_eq!(
            config.configured_pins[1].1,
            PinFunction::Input(Some(PullUp))
        );
    }

    #[test]
    fn save_one_pin_config_output_with_level() {
        let config = GPIOConfig {
            configured_pins: vec![(7, PinFunction::Output(Some(true)))], // GPIO7 output set to 1
        };

        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.piggui");

        config.save(test_file.to_str().unwrap()).unwrap();

        let pin_config = r#"{"configured_pins":[[7,{"Output":true}]]}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
    }
}
