# 安装指南

## 一键安装

```bash
curl -fsSL https://raw.githubusercontent.com/UA-Jin/CradleRing/main/install.sh | bash
```

## 手动安装

```bash
git clone https://github.com/UA-Jin/CradleRing.git
cd CradleRing
cargo build --release --bin cradle-ring
./install.sh
cradle-ring gateway start
```

## 访问

浏览器打开 http://127.0.0.1:18800
