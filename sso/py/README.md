# SSO

为了使 README 文档更加详尽和具体，我们可以加入更多关于功能的描述以及系统是如何实现这些功能的详细信息。以下是加强后的文档草稿：

---

# Single Sign-On (SSO) Services

## 介绍
这个项目实现了一个基于 Flask 和 JWT 的单点登录（SSO）系统，包含两个主要部分：SSO 认证服务和应用接入。SSO 认证服务负责处理用户的登录请求、生成和验证令牌；应用接入则利用这些令牌来校验用户的身份，确保只有验证通过的用户才能访问受保护的应用资源。

## 功能描述
- **用户认证**：用户可以通过用户名和密码进行登录，系统会验证凭据的有效性。
- **令牌生成和管理**：登录成功后，系统会生成一个 JWT 令牌，并将其返回给用户。令牌中包含了用户的基本信息和有效期。
- **令牌验证**：应用接入点会验证请求中携带的 JWT 令牌的有效性，以确认用户的身份。
- **会话管理**：支持用户的登录状态管理和令牌的失效处理。
- **安全防护**：实现了基本的安全措施，包括加密存储密码和 HTTPS 支持。

## 技术栈
- Python 3
- Flask
- JWT (JSON Web Tokens)
- Docker (可选，用于容器化部署)

## 安装指南
首先克隆仓库到本地：
```bash
git clone [仓库链接]
```
安装依赖：
```bash
cd [项目目录]
pip install -r requirements.txt
```

## 使用说明
### 启动 SSO 服务
```bash
cd sso_service
python app.py
```
该服务将运行在 `http://localhost:8000`。

### 启动应用接入服务
```bash
cd app_access
python app.py
```
该服务将运行在 `http://localhost:8001`。

## 配置
确保更新以下配置文件以匹配您的环境:
- `sso_service/config.py`
- `app_access/config.py`

## 依赖关系

这些可以通过 `pip install flask flask-talisman python-dotenv werkzeug pyjwt` 安装。

## 安全性考虑
- **JWT 密钥**：请确保在生产环境中使用安全的密钥。
- **HTTPS**：在生产环境中使用 HTTPS 来增加通信的安全性。
- **数据存储**：用户信息和令牌应存储在安全的环境中。

## 贡献
欢迎通过 Pull Requests 或 Issues 来贡献您的改进。

## 许可证
此项目采用 [MIT 许可证](LICENSE)。

