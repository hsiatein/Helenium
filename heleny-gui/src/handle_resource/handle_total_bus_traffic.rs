use crate::FrontendHandler;
use crate::terminal::generate_svg_path;
use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use std::collections::VecDeque;

impl FrontendHandler {
    pub async fn handle_total_bus_traffic(
        &self,
        data: VecDeque<(DateTime<Local>, usize)>,
    ) -> Result<()> {
        let (svg, y_max, y_mid, x_start, x_end) = generate_svg_path(&data, 600., 240.)?;
        self.ui_weak
            .upgrade_in_event_loop(move |ui| {
                ui.set_bus_stats_chart(svg.into());
                ui.set_bus_y_max(y_max.into());
                ui.set_bus_y_mid(y_mid.into());
                ui.set_bus_x_start(x_start.into());
                ui.set_bus_x_end(x_end.into());
            })
            .context("绘图 bus_stats_chart 失败")?;
        Ok(())
    }
}
