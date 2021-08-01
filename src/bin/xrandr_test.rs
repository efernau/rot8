use rot8::devices::xrandr::XRandRCreator;
use rot8::devices::{Rotator, RotatorCreator};
use rot8::orientation::AbsoluteOrientation;

use async_std::task::sleep;
use std::time::Duration;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    println!("Hi.");
    let driver = XRandRCreator {};
    for mon in driver.available_rotators().await.unwrap() {
        println!("aileron rolling {}!!", mon);
        let rotator = driver.create_rotator(mon.clone()).await.unwrap();

        rotator
            .set_orientation(AbsoluteOrientation::RightUp)
            .await
            .unwrap();
        println!(
            "current orientation of {}: {:?}",
            mon,
            rotator.get_current_orientation().await.unwrap()
        );
        sleep(Duration::from_secs(1)).await;
        rotator
            .set_orientation(AbsoluteOrientation::Flipped)
            .await
            .unwrap();
        println!(
            "current orientation of {}: {:?}",
            mon,
            rotator.get_current_orientation().await.unwrap()
        );
        sleep(Duration::from_secs(1)).await;
        rotator
            .set_orientation(AbsoluteOrientation::LeftUp)
            .await
            .unwrap();
        println!(
            "current orientation of {}: {:?}",
            mon,
            rotator.get_current_orientation().await.unwrap()
        );
        sleep(Duration::from_secs(1)).await;
        rotator
            .set_orientation(AbsoluteOrientation::Normal)
            .await
            .unwrap();
        println!(
            "current orientation of {}: {:?}",
            mon,
            rotator.get_current_orientation().await.unwrap()
        );
        sleep(Duration::from_secs(1)).await;
    }
    println!("Bye.");
    Ok(())
}
