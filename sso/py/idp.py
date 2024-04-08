from datetime import datetime, timedelta
from uuid import uuid4
from flask import Flask, jsonify, request, redirect, make_response

app = Flask(__name__)

TOKEN_EXPIRATION_MINUTES = 30

tokens = {}
# 存储用户名和密码（示例用，实际应用中应使用数据库）
users = {"user1": "password1"}


@app.route("/login", methods=["GET", "POST"])
def login():
    # 如果是POST请求，尝试执行登录逻辑
    if request.method == "POST":
        username = request.form.get("username")
        password = request.form.get("password")
        redirect_url = request.args.get("redirect_url", "")

        if users.get(username) == password:  # 简单的认证逻辑
            token = str(uuid4())  # 实际应用中应生成唯一Token
            token_issue_time = datetime.now()
            tokens[token] = {"username": username, "issue_time": token_issue_time}
            # 登录成功后，重定向到之前请求的服务
            response = make_response(redirect(redirect_url))
            response.set_cookie(
                "auth_token", token, httponly=True, secure=True
            )  # 在生产环境中使用secure=True
            # 生成响应
            # response = jsonify({'message': 'You are logged in successfully.', 'token': token})
            return response
        else:
            return "Login Failed! Invalid username or password.", 401

    # 如果是GET请求，展示登录表单
    return """
        <form method="post">
            Username: <input type="text" name="username"><br>
            Password: <input type="password" name="password"><br>
            <input type="submit" value="Login">
        </form>
    """


@app.route("/verify", methods=["GET"])
def verify():
    # token = request.args.get('token', '')
    token = request.headers.get("Authorization", None)
    token_info = tokens.get(token)

    if not token_info:
        return jsonify({"status": "invalid"}), 403

    if datetime.now() - token_info["issue_time"] > timedelta(minutes=TOKEN_EXPIRATION_MINUTES):
        # Token过期
        del tokens[token]  # 清理过期Token
        return jsonify({"status": "expired"}), 403

    # Token有效
    return jsonify({"status": "valid", "username": token_info["username"]})


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8000)
