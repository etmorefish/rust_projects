from functools import wraps
from flask import Flask, g, request, redirect, jsonify
import requests


app = Flask(__name__)
# 假设用户信息存储
users_info = {
    "user1": {"email": "user1@example.com", "password": "123", "name": "User 1"},
    "user2": {"email": "user2@example.com", "password": "123", "name": "User 2"},
}

JWT_SECRET = "sso-3E0C07FFFCFFF3E00E0039FCE00E7F387"  # 加解密密钥
JWT_ALGORITHM = "HS256"  # 加解密算法


def token_required(f):
    @wraps(f)
    def decorated(*args, **kwargs):
        token = request.cookies.get("auth_token")
        if not token:
            return redirect("http://localhost:8000/login?redirect_url=http://localhost:8002")

        headers = {'Authorization': token}
        response = requests.post(
            'http://localhost:8000/verify', headers=headers)
        if response.status_code == 200 and response.json()['status'] == 'valid':
            g.username = response.json().get('username')
        else:
            return redirect("http://localhost:8000/login?redirect_url=http://localhost:8002")
        return f(*args, **kwargs)
    return decorated


@app.route("/logout")
@token_required
def logout():
    token = request.cookies.get("auth_token")
    # 向认证中心发送登出请求
    headers = {"Authorization": token}
    response = requests.get("http://localhost:8000/logout", headers=headers)

    if response.status_code == 200:
        return f"Logout successfully!"
    else:
        # Token无效，重定向到登录页面
        return redirect(
            "http://localhost:8000/login?redirect_url=http://localhost:8002"
        )


@app.route("/")
@token_required
def home():
    username = g.username
    return f"This is app2, Welcome {username}! You are logged in."


@app.route("/profile")
@token_required
def profile():
    username = g.username
    user_info = users_info.get(username, {})
    return jsonify(user_info)


@app.route("/change-password", methods=["POST"])
def change_password():
    token = request.args.get("token", None)
    new_password = request.form.get("new_password", None)

    if token and new_password:
        # 验证Token
        response = requests.get(
            "http://localhost:8000/verify", params={"token": token})
        if response.status_code == 200:
            username = response.json().get("username")
            # 简化示例，直接更新内存中的密码
            if username in users_info:
                users_info[username]["password"] = new_password
                return jsonify({"message": "Password changed successfully."})
            else:
                return jsonify({"error": "User not found."}), 404
        else:
            return jsonify({"error": "Invalid or expired token."}), 403
    else:
        return jsonify({"error": "Missing token or new password."}), 400


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8002)
