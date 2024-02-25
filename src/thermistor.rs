//thermistor.rs 

pub const LUT_NTU_CELL_MODULE_D: &'static [f32] = &[
    74.89, 71.1, 67.53, 64.16, 60.98, 57.98, 55.15, 52.48, 49.95, 47.57, 45.31, 43.18, 41.16, 39.24, 37.43,
    35.72, 34.09, 32.55, 31.09, 29.7, 28.38, 27.13, 25.94, 24.81, 23.74, 22.72, 21.75, 20.83, 19.95, 19.12,
    18.32, 17.57, 16.84, 16.16, 15.5, 14.88, 14.28, 13.71, 13.17, 12.65, 12.16, 11.69, 11.24, 10.81, 10.39,
    10.0, 9.623, 9.263, 8.918, 8.588, 8.272, 7.97, 7.68, 7.402, 7.136, 6.881, 6.636, 6.402, 6.177, 5.961, 
    5.754, 5.555, 5.365, 5.182, 5.006, 4.837, 4.674, 4.518, 4.368, 4.224, 4.085, 3.952, 3.823, 3.7, 3.581, 
    3.466, 3.356, 3.25, 3.148, 3.05, 2.955, 2.863, 2.775, 2.691, 2.609, 2.53, 2.454, 2.38, 2.309, 2.241, 
    2.174, 2.111, 2.049, 1.989, 1.931, 1.876, 1.822, 1.77, 1.72, 1.671, 1.624,
];


pub struct Thermistor {
    //t0: f32,          // 25°C
    vreg: f32,        // 3V
    rntc: f32,        // 10k resistor for voltage bridge
    //b: u16,           // Thermistor constant
    min_temp: f32,    // lower temp limit
    max_temp: f32,    // upper temp limit
    temp_increment: f32,  // temperature increment in LUT
    lut_length: u16,    // length of LUT
    lut: &'static [f32], // pointer to lut
}

impl Thermistor {
    pub fn new(vreg: f32, rntc: f32, min_temp: f32, max_temp: f32, temp_increment: f32, lut_length: u16, lut: &'static [f32]) -> Self {
        Thermistor {
            //t0,
            vreg,
            rntc,
            //b,
            min_temp,
            max_temp,
            temp_increment,
            lut_length,
            lut,
        }
    }

    

    pub fn convert_volt_to_temp(&self, thermistor_voltage: u16) -> i16 {
        // Extracting the thermistor value in kOhm from the voltage divider bridge
        let r_therm = ((f32::from(thermistor_voltage) / 10000.0 / self.vreg) * self.rntc) / (1.0 - (f32::from(thermistor_voltage) / 10000.0 / self.vreg));

        let temp_increment_in_lut = self.temp_increment; // Required for LUTs that have different than 1°C increment (+5°C for i.e)
                                                          // Can be computed : (maxTemp-minTemp)/(lutLength-1)
        let mut temperature = self.min_temp; // Offsetting initial temperature (based on LUT)
        if r_therm < self.lut[self.lut_length as usize - 1] { // Temperature not in LUT
            temperature = self.max_temp; // Higher than maximal temperature
        } else if r_therm > self.lut[0] {
            temperature = self.min_temp; // Lower than minimal temperature
        } else {
            // Browsing the look up table
            for r_iterator in 0..self.lut_length {
                if r_therm >= self.lut[r_iterator as usize] {
                    if r_iterator > 0 {
                        // We can approximate a float value for the temperature
                        temperature += f32::from(r_iterator - 1) * temp_increment_in_lut; // Integer value for the temperature
                        // Linear approximation 
                        temperature += temp_increment_in_lut * (1.0 - (r_therm - self.lut[r_iterator as usize]) / (self.lut[r_iterator as usize - 1] - self.lut[r_iterator as usize]));
                    }
                    break;
                }
            }
        }

        (temperature * 10.0) as i16 // i.e : 39.6 °C ==> 396
    }
}

