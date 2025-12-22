<a id="readme-top"></a>



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/hsiatein/Helenium">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
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

### AI使用情况

1. heleny-utils::init_tracing函数 (1-31行) 为AI生成.
2. heleny-kernel::service::test_order的所有单元测试 (1-117行) 为AI生成.

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>


<!-- GETTING STARTED -->
## 开始

### 预先准备

所需要的软件.
* Rust

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
3. 新建.env文件, 在里面设置HELENIUM_CONFIG环境变量
   ```
   touch ./.env
   ```
4. 运行
   ```sh
   ./target/release/heleny_server
   ```

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- USAGE EXAMPLES -->
## 用法

更多例子请参考 [文档](https://example.com)

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- ROADMAP -->
## 路线

- [ ] 初步做出可以对话的原型

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>


<!-- LICENSE -->
## License

Distributed under the Unlicense License. See `LICENSE.txt` for more information.

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
* [Malven's Flexbox Cheatsheet](https://flexbox.malven.co/)
* [Malven's Grid Cheatsheet](https://grid.malven.co/)
* [Img Shields](https://shields.io)
* [GitHub Pages](https://pages.github.com)
* [Font Awesome](https://fontawesome.com)
* [React Icons](https://react-icons.github.io/react-icons/search)

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>