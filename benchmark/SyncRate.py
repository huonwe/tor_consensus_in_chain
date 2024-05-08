import requests
import time
import threading
from substrateinterface import SubstrateInterface, Keypair

# 根据情况调整
tor_ori_url = "http://localhost:7000/tor/status-vote/current/consensus.z"
blockchain_url = "http://localhost:8080/update"
substrate = SubstrateInterface(url="ws://127.0.0.1:9944")

FLAG_THREAD_CLOSE = False

def triger(t: float):
    print(f"同步间隔: {t}s")
    while not FLAG_THREAD_CLOSE:
        requests.get(blockchain_url)
        time.sleep(t)
    print("线程结束")

# 同步间隔
sync_intervals = [20,10,5,2,1,0.5]
# 检测间隔
check_intervals = [0.1,0.5,1,2,5]
print(">>> Start")
for c_interval in check_intervals:
    for s_interval in sync_intervals:
        FLAG_THREAD_CLOSE = False
        thread = threading.Thread(target=triger, args=(s_interval,))
        thread.start()
        # 测120s之内的正确率
        # 每隔c_interval秒检测一次, 一共检测 120 / c_interval 次
        total_count = 0
        # 对的次数
        hit_count = 0
        t0 = time.time()
        while time.time() - t0 < 120:
            ori_content = requests.get(tor_ori_url).text
            result = substrate.query("TemplateModule","Consensus")
            bc_content = result.value
            if ori_content == bc_content:
                hit_count += 1
            total_count += 1
            print("\r剩余时间: {:3.2f}s".format(120 - time.time() + t0),end='')
            time.sleep(c_interval)
        print("\r剩余时间: 000.00s")
        sync_rate = hit_count / total_count
        print(f"在同步间隔为{s_interval}s下, 每间隔{c_interval}s检测, 同步率为{sync_rate}")
        FLAG_THREAD_CLOSE = True
        thread.join()
print(">>> Done")