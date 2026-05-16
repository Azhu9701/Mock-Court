# 万民幡 Forgejo 自托管部署（Mac Mini 版）

## 前提

- Mac Mini（M1/M2/M3/M4），macOS 14+
- OrbStack 已安装（`brew install orbstack`）
- Tailscale 已安装（`brew install --cask tailscale`）
- Mac Mini 设为永不睡眠：`sudo pmset -a sleep 0 displaysleep 30`

## 1. 数据目录

放在用户目录下，OrbStack 对 `~/` 下的卷挂载性能最佳：

```bash
mkdir -p ~/ForgejoData/forgejo
cd ~/ForgejoData
```

把 `docker-compose.yml` 复制到 `~/ForgejoData/` 下。

## 2. 生成密钥

```bash
cd ~/ForgejoData
cat > .env <<EOF
SECRET_KEY=$(openssl rand -hex 32)
FORGEJO_DATA_DIR=$HOME/ForgejoData/forgejo
EOF
```

## 3. 启动

```bash
# OrbStack 自动启动 Docker daemon
docker compose up -d

# 看日志
docker compose logs -f
```

首次访问 `http://localhost:3000`，设置管理员账号。
之后 `DISABLE_REGISTRATION` 保持 `true`——新用户由管理员手动创建。

## 4. Tailscale 组网

### Mac Mini 端

```bash
brew install --cask tailscale
open /Applications/Tailscale.app
# 登录，给这台机器命名为 "forgejo"
```

设置里勾选：
- **MagicDNS**: 开启
- **Run unattended**: 让 Tailscale 后台常驻

### 你的开发机/其他成员设备

同样装 Tailscale 并登录到同一账号/团队。

### 访问

之后所有设备直接用 `http://forgejo:3000` 访问——MagicDNS 自动解析。
SSH 也通：`ssh git@forgejo -p 2222`。

## 5. 迁移仓库（GitHub → Forgejo）

```bash
# 1. 在 Forgejo Web UI 创建组织 "wanminfan" 和空仓库 "soul-banner-pro"

# 2. 镜像克隆 GitHub 仓库
cd /tmp
git clone --mirror git@github.com:Azhu97/soul-banner-pro.git

# 3. 推到 Forgejo（用 SSH key 或 token）
cd soul-banner-pro.git
git remote set-url --push origin http://forgejo:3000/wanminfan/soul-banner-pro.git
git push --mirror

# 4. 验证后删除 GitHub 仓库或设为 private
```

LFS 资源（如果有）：

```bash
git lfs fetch --all
git lfs push --all origin
```

## 6. 开发工作流切换

```bash
# 本地 banner 仓库改 remote
cd "/Users/huyi/Desktop/rust banner"
git remote set-url origin http://forgejo:3000/wanminfan/soul-banner.git

# 内部魂目录从 Forgejo clone
git clone http://forgejo:3000/wanminfan/souls-internal.git ~/souls-internal

# 启动时指向它
export WANMINFAN_SOULS_INTERNAL_DIR=~/souls-internal
```

可以把 `WANMINFAN_SOULS_INTERNAL_DIR` 加到 `~/.zshrc` 里持久化。

## 7. 备份

Mac Mini 内置 Time Machine 已经够用，但 Forgejo 推荐 dump：

```bash
# 手动备份
docker exec forgejo forgejo dump -c /data/forgejo/conf/app.ini

# 自动备份（launchd plist）
cat > ~/Library/LaunchAgents/com.wanminfan.forgejo-backup.plist <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
  <key>Label</key><string>com.wanminfan.forgejo-backup</string>
  <key>ProgramArguments</key>
  <array>
    <string>/bin/sh</string>
    <string>-c</string>
    <string>docker exec forgejo forgejo dump -c /data/forgejo/conf/app.ini --tempdir /data</string>
  </array>
  <key>StartCalendarInterval</key>
  <dict><key>Hour</key><integer>3</integer><key>Minute</key><integer>0</integer></dict>
</dict>
</plist>
EOF
launchctl load ~/Library/LaunchAgents/com.wanminfan.forgejo-backup.plist
```

每天凌晨 3 点跑一次 dump，产物在 `~/ForgejoData/forgejo/` 下。
配合 Time Machine 自动多版本归档。

## 8. 关键决策回顾

| 决策 | 选定 | 原因 |
|---|---|---|
| 数据库 | SQLite | 单组织/小团队够用，零运维 |
| 平台 | Forgejo 9 (arm64) | 社区治理，M 芯片原生 |
| 容器 | OrbStack | 比 Docker Desktop 快 3-5x，免费版够用 |
| 组网 | Tailscale + MagicDNS | 零端口暴露，外部完全不可达 |
| 备份 | dump + Time Machine | 双保险 |
| 注册 | 关闭 | 管理员手动审核开号 |

## 9. 安全清单

- [ ] Mac Mini 全盘 FileVault 加密
- [ ] OrbStack 设置开机自启
- [ ] Forgejo 管理员账号用 2FA
- [ ] Tailscale ACL 限制 forgejo 机器只能被特定 tag 访问
- [ ] 定期 `docker compose pull && docker compose up -d` 升级 Forgejo
- [ ] 公网 IP 不要做端口转发——一切走 Tailscale