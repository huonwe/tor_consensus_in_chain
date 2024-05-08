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
sync_intervals = [0.5]
print(">>> Start")
for s_interval in sync_intervals:
    FLAG_THREAD_CLOSE = False
    thread = threading.Thread(target=triger, args=(s_interval,))
    thread.start()
    # 新共识出现次数
    total_count = 0
    # 出现的延迟
    delay_accumulate = 0
    for index in range(10):
        old_ori_content = None
        old_bc_content = None
        time_ori_change = None
        time_bc_change = None
        flag_ori_changed = False
        flag_bc_changed = False
        time_start = time.time()
        while (time.time() - time_start) < 30:
            # print(flag_ori_changed, flag_bc_changed)
            ori_content = requests.get(tor_ori_url).text
            result = substrate.query("TemplateModule","Consensus")
            bc_content = result.value
            if ori_content != old_ori_content and not flag_ori_changed:
                if old_ori_content == None:
                    old_ori_content = ori_content
                else:
                    old_ori_content = ori_content
                    time_ori_change = time.time()
                    flag_ori_changed = True
            if bc_content != old_bc_content and flag_ori_changed and (not flag_bc_changed):
                if old_bc_content == None:
                    old_bc_content = bc_content
                else:
                    old_bc_content = bc_content
                    if bc_content == ori_content:
                        time_bc_change = time.time()
                        flag_bc_changed = True
            print(f"\r{index}, {flag_bc_changed}, {flag_ori_changed}", end="")
            time.sleep(0.2)
            if flag_bc_changed and flag_ori_changed:
                break
        if flag_bc_changed and flag_ori_changed:
            delay = time_bc_change - time_ori_change
            print("\ndelay:",delay)
            total_count += 1
            delay_accumulate += delay
            time.sleep(1)
        else:
            print("failed")
    delay_avg = delay_accumulate / total_count
    print(f"在同步间隔为{s_interval}s下, 总延迟为{delay_accumulate}s, 平均延迟为{delay_avg}")
    FLAG_THREAD_CLOSE = True
    thread.join()
print(">>> Done")