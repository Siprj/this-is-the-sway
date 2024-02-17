use std::process::exit;

use eyre::OptionExt;
use swayipc::{Connection, Event, EventType};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn select_workspace(
    connection: &mut Connection,
    target_name: String,
) -> color_eyre::eyre::Result<()> {
    let workspaces = connection.get_workspaces()?;
    let target_workspace = workspaces
        .iter()
        .find(|workspace| workspace.name == target_name);

    match target_workspace {
        Some(target_workspace) => {
            let focused_workspace = workspaces
                .iter()
                .find(|workspace| workspace.focused)
                .ok_or_eyre("no focused workspace")?;
            if target_workspace.output == focused_workspace.output {
                let command = format!("workspace {}", target_name);
                debug!("executing command: {}", &command);
                connection.run_command(command)?;
            } else if target_workspace.visible {
                let target_name = &target_workspace.name;
                let target_output = &target_workspace.output;
                let focused_name = &focused_workspace.name;
                let focused_output = &focused_workspace.output;
                let command = format!(
                    "[workspace={target_name}] move workspace to output {focused_output}; \
                [workspace={focused_name}] move workspace to output {target_output}; \
                workspace {target_name}"
                );
                debug!("executing command: {}", &command);
                connection.run_command(command)?;
            } else {
                let target_name = &target_workspace.name;
                let focused_output = &focused_workspace.output;
                let command =
                    format!("[workspace={target_name}] move workspace to output {focused_output};");
                debug!("executing command: {}", &command);
                connection.run_command(command)?;
            }
        }
        None => {
            let command = format!("workspace {}", target_name);
            debug!("executing command: {}", &command);
            connection.run_command(command)?;
        }
    }

    Ok(())
}

fn parse_command(command: &str) -> Option<String> {
    let command_parts: Vec<&str> = command.split(' ').collect();
    if command_parts.len() == 3
        && command_parts[0] == "nop"
        && command_parts[1] == "select_workspace"
    {
        return Some(command_parts[2].to_string());
    }
    None
}

fn main() -> color_eyre::eyre::Result<()> {
    let fmt_layer = fmt::layer().pretty();
    let filter_layer = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();
    color_eyre::install()?;

    let mut connection = Connection::new()?;
    for event in Connection::new()?.subscribe([EventType::Binding, EventType::Shutdown])? {
        match event {
            Ok(event) => match event {
                Event::Shutdown(_) => exit(0),
                Event::Binding(binding) => {
                    info!("biding event recevied");
                    let command = binding.binding.command;
                    let pars_result = parse_command(&command);
                    match pars_result {
                        Some(target) => {
                            info!("command sucessfully parsed target: {}", target);
                            select_workspace(&mut connection, target)?;
                        }
                        None => warn!("unknown command received: {}", command),
                    }
                }
                _ => error!("unrequested event received: {:?}", event),
            },
            Err(err) => {
                error!("subscribe failed with: {}", err);
            }
        }
    }
    Ok(())
}
