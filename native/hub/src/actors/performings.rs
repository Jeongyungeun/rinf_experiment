use async_trait::async_trait;
use messages::{
    actor::Actor,
    prelude::{Address, Context, Notifiable},
};
use rinf::RustSignalBinary;
use tokio::task::JoinSet;

use crate::signals::{SampleFractal, SampleSchema};

pub struct ImageInfo {
    pub scale: f64,
    pub data: Vec<u8>,
}
pub struct PerformingActor {
    _owned_tasks: JoinSet<()>,
}

impl Actor for PerformingActor {}

impl PerformingActor {
    pub fn new(self_addr: Address<Self>) -> Self {
        let mut owned_tasks = JoinSet::new();
        // owned_tasks.spawn(Self::run_debug_tests());
        // owned_tasks.spawn(Self::stream_fractal(self_addr));
        PerformingActor {
            _owned_tasks: owned_tasks,
        }
    }
}

#[async_trait]
impl Notifiable<ImageInfo> for PerformingActor {
    async fn notify(&mut self, msg: ImageInfo, _: &Context<Self>) {
        SampleFractal {
            current_scale: msg.scale,
            dummy: Some(SampleSchema {
                sample_field_one: true,
                sample_field_two: false,
            }),
        }
        .send_signal_to_dart(msg.data);
    }
}
impl PerformingActor {
    #[cfg(debug_assertions)]
    const IS_DEBUG_MODE: bool = true;

    #[cfg(not(debug_assertions))]
    const IS_DEBUG_MODE: bool = false;

    // async fn stream_fractal(mut self_addr:Address<>)
}
