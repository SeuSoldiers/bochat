### 需要做的社区化开发

- [ ] 开发Ncatbot插件，支持监控Bochat平台的群聊信息，并转发到QQ群/私聊中；支持监控QQ的群聊/私聊信息，并转发到Bochat平台。可能需要设计一套路由配置来让用户配置这个转发关系。（不需要UI，用户可以直接编辑配置文件来设置转发关系）
- [ ] 开发一个BOT用户使用的web界面，用户通过配置后端地址和token来登录，可以在界面上查看自己的群聊，在群聊中发送消息/文件，并查看收到的消息/文件。这个界面可以使用React/Vue等前端框架来开发。
- [ ] 开发一个BOT使用的订阅器，可以订阅 Github 上的某个仓库的各类事件（如issue、pull request等），当有事件发生时，自动将事件信息转发到指定的群聊中。用户可以通过配置文件来设置订阅的仓库和事件类型，以及转发的群聊。
- [ ] 开发一个BOT使用的订阅器，可以订阅RSS源，当有新的内容发布时，自动将内容信息转发到指定的群聊中。用户可以通过配置文件来设置订阅的RSS源，检测频率以及转发的群聊。并且支持通过命令手动查询。
- [ ] 开发一个BOT翻译功能，用户可以通过`/trans <c2e/e2c> <文本>`命令来翻译文本，支持中英文即可。可以使用第三方翻译API（如Google翻译、百度翻译等）来实现翻译功能。 
- [ ] 开发一个BOT使用的定时任务功能，用户可以通过配置文件来设置定时任务，例如每天定时发送一条消息到指定的群聊中，或者每周定时检查某个网站的更新并将更新信息转发到群聊中。


### Tips

尽量基于仓库中的rust-sdk或python-sdk来开发插件，避免直接调用HTTP API，这样可以更好地利用SDK提供的功能和抽象层，减少开发难度和维护成本。


社区项目保持**完全独立仓库**维护；主仓库只维护一个**索引文件**，不托管社区项目源码。  
推荐做法：主仓库提交 `YAML` 索引，社区仓库维护自己的 `README.md` 与发布版本。

主仓库 PR 只包含：新增或更新 `community/proj-name.yaml` 中对应条目

每个社区仓库至少包含以下内容：

1. `README.md`：功能简介、安装方式、配置方式、最小示例
2. `LICENSE`


示例：`community/index.yaml`

```yaml
id: ncat-bochat-bridge
name: Ncat BoChat Bridge
status: active
summary: QQ 与 BoChat 的双向消息转发插件
repo: https://github.com/example/ncat-bochat-bridge
homepage: https://github.com/example/ncat-bochat-bridge#readme
maintainer: "@alice"
sdk: python
tags: [qq, bridge, relay]
```

字段约定：

- `id`: 全局唯一，建议 kebab-case，后续不变更
- `name`: 展示名
- `status`: `active` / `beta` / `deprecated`
- `summary`: 一句话描述
- `repo`: 仓库地址
- `homepage`: 文档入口，通常指向 `README`
- `maintainer`: 维护者标识
- `tags`: 检索标签