use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::{PhysicalPosition, PhysicalRect, PhysicalSize};

const FILE: &str = "bar.json";

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Kept {
    spots: HashMap<String, Spot>,
}

pub fn spot_of(base: &Path, screen: &str) -> Option<Spot> {
    crate::store::stored::<Kept>(&base.join(FILE)).spots.get(screen).copied()
}

pub fn keep(base: &Path, screen: &str, spot: Option<Spot>) -> Result<(), String> {
    crate::store::update(base, FILE, |kept: &mut Kept| {
        match spot {
            Some(spot) => kept.spots.insert(screen.to_string(), spot),
            None => kept.spots.remove(screen),
        };
        Ok(())
    })
}

pub fn offset(area: &PhysicalRect<i32, u32>, scale: f64, at: PhysicalPosition<i32>) -> Spot {
    Spot {
        x: f64::from(at.x - area.position.x) / scale,
        y: f64::from(at.y - area.position.y) / scale,
    }
}

pub fn settled(area: &PhysicalRect<i32, u32>, scale: f64, spot: Spot, size: PhysicalSize<u32>) -> PhysicalPosition<i32> {
    let within = |start: i32, span: u32, wide: u32, along: f64| {
        let wanted = (f64::from(start) + along * scale).round() as i32;
        wanted.min(start + span as i32 - wide as i32).max(start)
    };
    PhysicalPosition::new(
        within(area.position.x, area.size.width, size.width, spot.x),
        within(area.position.y, area.size.height, size.height, spot.y),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(x: i32, y: i32, width: u32, height: u32) -> PhysicalRect<i32, u32> {
        PhysicalRect {
            position: PhysicalPosition::new(x, y),
            size: PhysicalSize::new(width, height),
        }
    }

    #[test]
    fn a_spot_is_kept_in_the_screen_s_own_pixels_and_comes_back_where_it_was() {
        let second = area(1920, 0, 2560, 1400);
        let at = PhysicalPosition::new(2420, 300);
        let spot = offset(&second, 1.25, at);
        assert_eq!(spot, Spot { x: 400.0, y: 240.0 });
        assert_eq!(settled(&second, 1.25, spot, PhysicalSize::new(860, 132)), at);
    }

    #[test]
    fn a_spot_never_leaves_the_bar_partly_off_the_screen() {
        let small = area(0, 0, 1280, 720);
        let size = PhysicalSize::new(688, 106);
        assert_eq!(settled(&small, 1.0, Spot { x: 1200.0, y: 700.0 }, size), PhysicalPosition::new(592, 614));
        assert_eq!(settled(&small, 1.0, Spot { x: -50.0, y: -20.0 }, size), PhysicalPosition::new(0, 0));
    }

    #[test]
    fn each_screen_keeps_its_own_spot_until_it_is_forgotten() {
        let base = std::env::temp_dir().join(format!("sens-spots-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        assert_eq!(spot_of(&base, r"\\.\DISPLAY1"), None);

        keep(&base, r"\\.\DISPLAY1", Some(Spot { x: 10.0, y: 20.0 })).unwrap();
        keep(&base, r"\\.\DISPLAY2", Some(Spot { x: 30.0, y: 40.0 })).unwrap();
        assert_eq!(spot_of(&base, r"\\.\DISPLAY1"), Some(Spot { x: 10.0, y: 20.0 }));
        assert_eq!(spot_of(&base, r"\\.\DISPLAY2"), Some(Spot { x: 30.0, y: 40.0 }));

        keep(&base, r"\\.\DISPLAY1", None).unwrap();
        assert_eq!(spot_of(&base, r"\\.\DISPLAY1"), None);
        assert_eq!(spot_of(&base, r"\\.\DISPLAY2"), Some(Spot { x: 30.0, y: 40.0 }));
        let _ = std::fs::remove_dir_all(&base);
    }
}
