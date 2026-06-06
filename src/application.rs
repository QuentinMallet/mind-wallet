use crate::{commands::EntryPoint, config::AppConfig};
use abscissa_core::{
    Application, FrameworkError, StandardPaths, application,
    component::Component,
    config::{self, CfgCell},
};

pub static APPLICATION: application::AppCell<MindWalletApplication> = application::AppCell::new();

#[derive(Debug, Default)]
pub struct MindWalletApplication {
    config: CfgCell<AppConfig>,
    state: application::State<Self>,
}

impl Application for MindWalletApplication {
    type Cmd = EntryPoint;
    type Cfg = AppConfig;
    type Paths = StandardPaths;

    fn config(&self) -> config::Reader<AppConfig> {
        self.config.read()
    }

    fn state(&self) -> &application::State<Self> {
        &self.state
    }

    fn register_components(&mut self, command: &Self::Cmd) -> Result<(), FrameworkError> {
        let framework_components: Vec<Box<dyn Component<Self>>> =
            self.framework_components(command)?;
        self.state.components_mut().register(framework_components)
    }

    fn after_config(&mut self, config: Self::Cfg) -> Result<(), FrameworkError> {
        self.state.components_mut().after_config(&config)?;
        self.config.set_once(config);
        Ok(())
    }
}
