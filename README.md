<a id="readme-top"></a>



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/hsiatein/Helenium">
    <img src="assets\icon.png" alt="Logo" width="80" height="80">
  </a>

  <h3 align="center">Helenium</h3>

  <p align="center">
    基于Rust的智能体项目.
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>目录</summary>
  <ol>
    <li>
      <a href="#关于项目">关于项目</a>
      <ul>
        <li><a href="#ai使用情况">AI使用情况</a></li>
      </ul>
    </li>
    <li>
      <a href="#开始">开始</a>
      <ul>
        <li><a href="#预先准备">预先准备</a></li>
        <li><a href="#安装">安装</a></li>
      </ul>
    </li>
    <li><a href="#用法">用法</a></li>
    <li><a href="#路线">路线</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#联系">联系</a></li>
    <li><a href="#致谢">致谢</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## 关于项目

### 项目架构

本项目基于Actor架构.

项目分为内核(Kernel), 各个服务(Service), 启动的时候内核将拉起内核服务(KernelService), 而后内核服务负责拉起后续所有服务.

所有服务可以有自己的依赖, 其中所有服务都默认依赖内核服务, 并被内核服务管理生命周期. 

GUI的实现基于 Slint(当前), WEBUI的实现基于 Vue/TS(暂时停更)

### AI使用情况

本项目测试大多数使用AI生成, 在测试以外使用AI生成的还有:
1. heleny-utils\src\lib.rs::init_tracing函数 (6-31行).
2. heleny-macros\src\lib.rs::base_service宏 (1-63行).
3. 大部分 WebUI 代码 ( *.vue, *.ts ).
4. *.slint文件.
5. heleny-gui/src/terminal.rs的svg生成 (7-123行).
6. schedule 中 chrono 的用法. 
7. SQL相关.

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>


<!-- GETTING STARTED -->
## 开始

### 预先准备

所需要的软件.

构建：
* Rust工具链
* cmake ( 大概可能是gui中的 Slint 库编译要用 )

MCP工具 ( 不安装不影响使用聊天，但是影响使用MCP工具 ) ：
* Docker
* npm
* uv

### 安装

1. 克隆仓库
   ```sh
   git clone https://github.com/hsiatein/Helenium.git
   cd Helenium
   ```
2. 构建
   ```sh
   cargo build --release
   ```
3. 在项目根目录新建.env文件, 在里面设置HELENIUM_CONFIG环境变量
   ```
   HELENIUM_CONFIG=./Config.json
   LAUNCH_HELENIUM_BACKEND=true
   XXX_API_KEY=xxx...
   ```
   
4. LAUNCH_HELENIUM_BACKEND=true时，只需要运行GUI，GUI会自动拉起服务端
   ```sh
   ./target/release/heleny_gui
   ```

5. LAUNCH_HELENIUM_BACKEND=false时，客户端和服务端分离，需要先拉起服务端再运行GUI
   ```sh
   ./target/release/heleny_server
   ./target/release/heleny_gui
   ```

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- USAGE EXAMPLES -->
## 用法

UI 服务默认开在4080端口，可以在config.json->WebuiService->port修改

config.json->ChatService->api是可用的api的数组，其中api密钥填环境变量名，具体值由环境变量值给出

config.json->ChatService->heleny/planner/executor->api是api数组的索引，表示使用哪一个api



可以创建assets/presets/persona.txt文件，写入人设。

将mcp服务器的启动command放入script/mcp.json，并运行script/src/bin/mcp_tools.rs可以把mcp的工具列表转化为Helenium的工具说明书（放在script目录），然后把说明书放assets/tools，启动command放Config.json的McpService.mcp_servers里面，即可增加新的mcp工具。

暂时没有文档.

更多例子请参考 [文档](https://example.com)

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- ROADMAP -->
## 路线

- [x] 初步做出可以对话，可以使用工具完成任务的原型

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>


<!-- LICENSE -->
## License

基于 GPLv3 许可证发布. 更多信息见 `LICENSE` .

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- CONTACT -->
## 联系

牛奶小麦 - mugilovemilk@mail.ustc.edu.cn

项目链接: [https://github.com/hsiatein/Helenium](https://github.com/hsiatein/Helenium)

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## 致谢

* 异步运行时: [Tokio](https://tokio.rs/)
* 日志: [Tracing](https://github.com/tokio-rs/tracing)
* GUI: [Slint](https://github.com/slint-ui/slint)

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>