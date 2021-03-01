# mpt-get

mpt-get 是一款管理 mirai-console 插件包的工具，类似于 apt-get 和 mcl

## 如何使用

### 更新索引

使用命令：

```bash
mpt-get update
```

来拉取索引，默认源是 Gitee (https://gitee.com/peratx/mirai-repo.git)。

~~可以在配置文件中修改源~~ 还没做

### 列出所有可用包

使用命令：

```bash
mpt-get list
```

可以列出所有可用包（如果没有拉取索引会报错：找不到 packages.json）。