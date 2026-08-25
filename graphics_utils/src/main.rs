use std::f64::consts::PI;

// ========== 辅助 trait ==========
trait CosSin {
    fn cos_sin(&self) -> (f64, f64);
}

impl CosSin for f64 {
    fn cos_sin(&self) -> (f64, f64) {
        (self.cos(), self.sin())
    }
}

// ========== 四元数 ==========
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Quaternion {
    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { w, x, y, z }
    }

    /// 从欧拉角(ZYX 顺序)构造四元数
    pub fn from_euler(roll: f64, pitch: f64, yaw: f64) -> Self {
        let (cr, sr) = (roll * 0.5).cos_sin();
        let (cp, sp) = (pitch * 0.5).cos_sin();
        let (cy, sy) = (yaw * 0.5).cos_sin();
        Quaternion {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }

    /// 将四元数转为欧拉角(ZYX 顺序),返回 (roll, pitch, yaw)
    pub fn to_euler(&self) -> (f64, f64, f64) {
        let (w, x, y, z) = (self.w, self.x, self.y, self.z);

        let sinp: f64 = 2.0 * (w * y - z * x);

        if sinp >= 1.0 - 1e-12 {
            // 万向锁(+90°):roll 与 yaw 合并,固定 roll=0
            (0.0, PI / 2.0, -2.0 * x.atan2(w))
        } else if sinp <= -1.0 + 1e-12 {
            // 万向锁(-90°)
            (0.0, -PI / 2.0, 2.0 * x.atan2(w))
        } else {
            // 滚转 (roll)
            let sinr_cosp: f64 = 2.0 * (w * x + y * z);
            let cosr_cosp: f64 = 1.0 - 2.0 * (x * x + y * y);
            let roll: f64 = sinr_cosp.atan2(cosr_cosp);

            // 俯仰 (pitch)
            let pitch: f64 = sinp.asin();

            // 偏航 (yaw)
            let siny_cosp: f64 = 2.0 * (w * z + x * y);
            let cosy_cosp: f64 = 1.0 - 2.0 * (y * y + z * z);
            let yaw: f64 = siny_cosp.atan2(cosy_cosp);

            (roll, pitch, yaw)
        }
    }
}

// ========== 欧拉角对象 ==========
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EulerAngles {
    pub roll: f64,  // 绕 X 轴
    pub pitch: f64, // 绕 Y 轴
    pub yaw: f64,   // 绕 Z 轴
}

impl EulerAngles {
    pub fn new(roll: f64, pitch: f64, yaw: f64) -> Self {
        Self { roll, pitch, yaw }
    }

    /// 转换为四元数(ZYX 顺序)
    pub fn to_quaternion(&self) -> Quaternion {
        Quaternion::from_euler(self.roll, self.pitch, self.yaw)
    }

    /// 从四元数转换(ZYX 顺序),注意万向锁情况
    pub fn from_quaternion(q: &Quaternion) -> Self {
        let (roll, pitch, yaw) = q.to_euler();
        EulerAngles { roll, pitch, yaw }
    }

    /// 便捷方法:直接使用 (roll, pitch, yaw) 元组构造
    pub fn from_tuple(angles: (f64, f64, f64)) -> Self {
        EulerAngles {
            roll: angles.0,
            pitch: angles.1,
            yaw: angles.2,
        }
    }

    /// 转为 (roll, pitch, yaw) 元组
    pub fn to_tuple(&self) -> (f64, f64, f64) {
        (self.roll, self.pitch, self.yaw)
    }
}

// ========== 测试 ==========
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quaternion_roundtrip() {
        let cases: Vec<(f64, f64, f64)> = vec![
            (0.1, 0.2, 0.3),
            (1.2, -0.5, 0.8),
            (-0.7, 1.5, -0.3),
            (0.0, 0.0, 0.0),
            (PI / 2.0, 0.0, 0.0),
            (0.0, PI / 2.0, 0.0),
            (0.0, 0.0, PI / 2.0),
        ];
        for (roll, pitch, yaw) in cases {
            let q: Quaternion = Quaternion::from_euler(roll, pitch, yaw);
            let (r2, p2, y2) = q.to_euler();
            // 常规情况下精确相等
            assert!((r2 - roll).abs() < 1e-12, "roll mismatch");
            assert!((p2 - pitch).abs() < 1e-12, "pitch mismatch");
            assert!((y2 - yaw).abs() < 1e-12, "yaw mismatch");
        }
    }

    #[test]
    fn test_euler_angles_object() {
        let original: EulerAngles = EulerAngles::new(0.5, -0.3, 1.2);
        let q: Quaternion = original.to_quaternion();
        let recovered: EulerAngles = EulerAngles::from_quaternion(&q);
        assert!((recovered.roll - original.roll).abs() < 1e-12);
        assert!((recovered.pitch - original.pitch).abs() < 1e-12);
        assert!((recovered.yaw - original.yaw).abs() < 1e-12);
    }

    #[test]
    fn test_gimbal_lock() {
        let euler: EulerAngles = EulerAngles::new(0.5, PI / 2.0, 0.7);
        let q: Quaternion = euler.to_quaternion();
        let recovered: EulerAngles = EulerAngles::from_quaternion(&q);

        let pitch_abs: f64 = recovered.pitch.abs();
        assert!(
            (pitch_abs - PI / 2.0).abs() < 1e-7,
            "pitch should be near ±PI/2, got {}",
            recovered.pitch
        );

        let q2: Quaternion = recovered.to_quaternion();
        let dot: f64 = q.w * q2.w + q.x * q2.x + q.y * q2.y + q.z * q2.z;
        assert!(
            dot.abs() > 0.9999,
            "quaternions are not equivalent, dot = {}",
            dot
        );
    }

    // 额外测试:边缘情况,sinp 略大于 1 的浮点误差
    #[test]
    fn test_near_gimbal_lock() {
        let pitch: f64 = PI / 2.0 - 1e-15; // 非常接近 90°
        let euler: EulerAngles = EulerAngles::new(0.1, pitch, 0.2);
        let q: Quaternion = euler.to_quaternion();
        let recovered: EulerAngles = EulerAngles::from_quaternion(&q);
        // 不应该 panic,且恢复的 pitch 应在合理范围
        assert!(
            (recovered.pitch - pitch).abs() < 1e-12
                || (recovered.pitch.abs() - PI / 2.0).abs() < 1e-10
        );
    }

    #[test]
    fn test_tuple_conversion() {
        let angles: (f64, f64, f64) = (0.1, 0.2, 0.3);
        let e: EulerAngles = EulerAngles::from_tuple(angles);
        assert_eq!(e.to_tuple(), angles);
    }
}

fn main() {}
