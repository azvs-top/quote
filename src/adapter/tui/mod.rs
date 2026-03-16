// TUI 模块采用适配 tuirealm 的 TEA 风格结构：
// - `state`：Model，保存持久化的界面状态。
// - `message`：Msg，定义输入层发出的消息。
// - `update`：Update，负责把 Msg 应用到 Model，并执行副作用。
// - `ui`：View，基于当前状态进行纯 ratatui 渲染。
// - `components`：tuirealm 适配层，负责输入转发与根组件渲染。
// - `program` / `runtime`：宿主程序与终端运行时集成。
mod components;
mod message;
mod program;
mod runtime;
mod state;
mod ui;
mod update;

pub use runtime::run;
