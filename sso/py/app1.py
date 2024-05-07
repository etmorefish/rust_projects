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
            return redirect("http://localhost:8000/login?redirect_url=http://localhost:8001")

        headers = {'Authorization': token}
        response = requests.post(
            'http://localhost:8000/verify', headers=headers)
        if response.status_code == 200 and response.json()['status'] == 'valid':
            g.username = response.json().get('username')
        else:
            return redirect("http://localhost:8000/login?redirect_url=http://localhost:8001")
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
            "http://localhost:8000/login?redirect_url=http://localhost:8001"
        )


@app.route("/")
@token_required
def home():
    username = g.username
    return f"This is app1, Welcome {username}! You are logged in."


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

"""
webhook_registered = False

def register_webhook(username):
    global webhook_registered
    data = {
        "username": username,  # 假设我们使用 'all' 表示注册一个Webhook用于所有用户的登出事件
        "webhook_url": "http://localhost:8002/logout_webhook"  # 应用处理登出Webhook的URL
    }
    response = requests.post("http://localhost:8000/register_webhook", json=data)
    if response.status_code == 200:
        webhook_registered = True
        print("Webhook registered successfully.")
    else:
        print("Failed to register webhook.")



def token_required(f):
    @wraps(f)
    def decorated(*args, **kwargs):
        token = request.cookies.get("auth_token")
        print("token: ", token)
        if not token:
            return redirect("http://localhost:8000/login?redirect_url=http://localhost:8002")


        # 注册
        if not webhook_registered:
            try:
                payload = jwt.decode(token, JWT_SECRET, algorithms=[JWT_ALGORITHM])
            except jwt.PyJWTError:
                return "Invalid Token.", 403
            username = payload.get("username")
            register_webhook(username)
            
        # 检查缓存
        cache_entry = token_cache.get(token)
        if cache_entry and cache_entry['expire_time'] > time.time():
            g.username = cache_entry['username']

        else:
            headers = {'Authorization': token}
            response = requests.post('http://localhost:8000/verify', headers=headers)
            if response.status_code == 200 and response.json()['status'] == 'valid':
                g.username = response.json().get('username')
                # 更新缓存，这里假设 token 有效期为 5 分钟
                token_cache[token] = {'username': g.username, 'expire_time': time.time() + 300}
            else:
                return redirect("http://localhost:8000/login?redirect_url=http://localhost:8002")
        return f(*args, **kwargs)
    return decorated
    
@app.route('/logout_webhook', methods=['POST'])
def logout_webhook():
    global webhook_registered 
    webhook_registered = False
    
    # 实际环境中，这里应该进行更安全的处理，例如验证Webhook来源
    print("Received logout webhook")
    print(1, request.json)
    token = request.cookies.get("auth_token")
    print(2, token)
    # 这里可以执行登出相关的清理逻辑
    response = make_response(jsonify({"message": "You have been logged out successfully."}))
    # 删除 cookie，具体通过设置cookie的过期时间为过去的时间
    response.delete_cookie('auth_token')
    
    token = request.cookies.get("auth_token")
    print(3, token)
    # return jsonify({"message": "Logout webhook received"}), 200
    return response
"""

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8001)
