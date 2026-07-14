#ifndef MEASUREMENT_H
#define MEASUREMENT_H
#include "string"
#include "Arduino.h"

namespace Measurement {



    namespace IR
    {
        constexpr int EMITTERS[4] = {IRLED_0, IRLED_1, IRLED_2, IRLED_3};
        constexpr int RECEIVERS[4] = {PD_0, PD_1, PD_2, PD_3};

        constexpr int CHANNEL_RIGHT = 2;
        constexpr int CHANNEL_LEFT = 1;
        constexpr int CHANNEL_FRONT1 = 0;
        constexpr int CHANNEL_FRONT2 = 3;

       inline int FinalDistances_Unconverted[4];
       inline int RawDistances_Unconverted[4];
       inline int Noises[4];

        void init();

        int getDistance(int channel);
        int getNoise(int channel);


        void refreshDistance(int channel);
      //  void refreshAllDistances();

        void refreshNoise(int channel);
     //   void refreshAllNoises();

        void debugPrintRawDistance(int channel);


        namespace Emitters {
            void initIR_LEDs();
            void enableLED(int channel);
            void disableLED(int channel);

        }

        namespace Receivers {
            void initPhotoSensors();
            int readDistance(int channel);
            int readAmbientNoise(int channel);
            
        }

        namespace calibration {
            inline int wallThresholdLeft;
            inline int wallThresholdRight;
            inline int wallThresholdFront;

            inline int absoluteWallThreshold;

            void calibrateWallThresholdLeft();
            void calibrateWallThresholdRight();
            void calibrateWallThresholdFront();

            void initCalibration(int calibrationSteps);

        }

        namespace WallDetection {
            constexpr int tolerancePercent = 0;


           inline bool isWallLeft;
            inline bool isWallRight;
            inline bool isWallFront;

            bool RefreshWallLeft();
            bool RefreshWallRight();
            bool RefreshWallFront();

            void RefreshAllWalls();

            void debugPrintWallDetectionStatus();

        }


    } 
    


    namespace Sensors {
    
    enum SensorNames {
        BQ76905_BATTERY_MANAGEMENT,
        FAN,
        TCAL6408_GPIO_EXPANDER_0,
        TCAL6408_GPIO_EXPANDER_1,
        LSM6DSR_ACCELEROMETER_GYROSKOP,
        TMP464_TEMPERATURE_SENSOR,
        TPL0102_POTENTIOMETER,
        VL53L4CD_TOF_0,
        VL53L4CD_TOF_1,
        VL53L4CD_TOF_2,
        IIS2MDC_MAGNETOMETER

    };

    enum SensorData {
    BATTERY_PERCENT_LEFT,

    
    FAN_SPEED,

    TOF_0_DISTANCE,
    TOF_1_DISTANCE,
    TOF_2_DISTANCE,

    IR_0_DISTANCE,
    IR_1_DISTANCE,
    IR_2_DISTANCE,
    IR_3_DISTANCE,

    TMP_TEMP_LOCAL,
    TMP_TEMP_REMOTE_1,
    TMP_TEMP_REMOTE_2,
    TMP_TEMP_REMOTE_3,
    TMP_TEMP_REMOTE_4,


    POT_WIPER_A,
    POT_WIPER_B,
    POT_VOLT_A,
    POT_VOLT_B,
    POT_DEFAULT_VOLT_A,
    POT_DEFAULT_VOLT_B, 

    ACC_X,
    ACC_Y,
    ACC_Z,
    GYRO_X,
    GYRO_Y,
    GYRO_Z

    };

    const char* to_string(SensorNames val);
    const char* to_string(SensorData val);
    uint8_t getI2CAddress(SensorNames Sensor);

    void sendSensorData(SensorData data, float value);

    void i_sensor(SensorNames SensorName, std::string infoMessage);
    void e_sensor(SensorNames SensorName, std::string errorMessage);
    void d_sensor(SensorNames SensorName, std::string debugMessage);

    void reportTemperature();
    void reportAcceleration();
    void reportGyroscope();
    void reportDistance();
    void reportBattery();
    



}
}



#endif