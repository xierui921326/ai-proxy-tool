import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type Settings = {
  listen: string;
  tls_listen?: string | null;
  openai_upstream?: string | null;
  anthropic_upstream?: string | null;
};

function App() {
  const [listen, setListen] = useState("127.0.0.1:8777");
  const [tlsListen, setTlsListen] = useState("127.0.0.1:9443");
  const [openai, setOpenai] = useState("https://api.openai.com");
  const [anthropic, setAnthropic] = useState("https://api.anthropic.com");
  const [appPath, setAppPath] = useState("");
  const [caPath, setCaPath] = useState("");
  const [logs, setLogs] = useState<string[]>([]);
  const [svcOn, setSvcOn] = useState(false);
  const [hostsStatus, setHostsStatus] = useState<"unknown" | "on" | "off">("unknown");

  const log = useMemo(() => (msg: string) => {
    setLogs((prev) => [...prev, `${new Date().toLocaleTimeString()} ${msg}`]);
  }, []);

  useEffect(() => {
    (async () => {
      try {
        const s = (await invoke("load_settings")) as Settings;
        if (s.listen) setListen(s.listen);
        if (s.tls_listen) setTlsListen(s.tls_listen);
        if (s.openai_upstream) setOpenai(s.openai_upstream);
        if (s.anthropic_upstream) setAnthropic(s.anthropic_upstream);
        setSvcOn(false);
        setHostsStatus("unknown");
        log("配置已加载");
      } catch (e: any) {
        log(`加载配置失败: ${String(e)}`);
      }
    })();
  }, [log]);

  const onStart = async () => {
    try {
      const cfg = {
        listen,
        tls_listen: tlsListen,
        openai_upstream: openai,
        anthropic_upstream: anthropic,
      };
      await invoke("start_proxy", { cfg });
      setSvcOn(true);
      log("代理服务已启动");
    } catch (e: any) {
      log(`启动失败: ${String(e)}`);
    }
  };

  const onApplyHosts = async () => {
    try {
      await invoke("apply_hosts");
      setHostsStatus("on");
      log("hosts 已接管");
    } catch (e: any) {
      log(`接管失败: ${String(e)}`);
    }
  };

  const onRevertHosts = async () => {
    try {
      await invoke("revert_hosts");
      setHostsStatus("off");
      log("hosts 已释放");
    } catch (e: any) {
      log(`释放失败: ${String(e)}`);
    }
  };

  const onExportCA = async () => {
    try {
      const p = (await invoke("export_root_ca_path")) as string;
      setCaPath(p);
      log(`根证书路径: ${p}`);
    } catch (e: any) {
      log(`导出失败: ${String(e)}`);
    }
  };

  const onSave = async () => {
    try {
      const cfg: Settings = {
        listen,
        tls_listen: tlsListen,
        openai_upstream: openai,
        anthropic_upstream: anthropic,
      };
      await invoke("save_settings", { cfg });
      log("配置已保存");
    } catch (e: any) {
      log(`保存失败: ${String(e)}`);
    }
  };

  const onClear = async () => {
    try {
      await invoke("clear_data");
      log("数据已清理");
    } catch (e: any) {
      log(`清理失败: ${String(e)}`);
    }
  };

  const onLaunch = async () => {
    try {
      const port = parseInt((listen.split(":")[1] || "8777"), 10);
      await invoke("launch_target", { appPath, port });
      log("已拉起目标应用");
    } catch (e: any) {
      log(`拉起失败: ${String(e)}`);
    }
  };

  return (
    <div style={{ padding: 16 }}>
      <h2 style={{ margin: "0 0 12px 0" }}>AI Proxy 控制台</h2>
      <div style={{ display: "flex", gap: 16, flexWrap: "wrap" }}>
        <div style={{ border: "1px solid #e5e7eb", borderRadius: 8, padding: 12, flex: 1, minWidth: 320 }}>
          <div style={{ fontWeight: 600, marginBottom: 8 }}>代理服务状态</div>
          <div style={{ display: "grid", gridTemplateColumns: "160px 1fr", gap: "8px 12px", alignItems: "center" }}>
            <label>HTTP 端口</label>
            <input value={listen} onChange={(e) => setListen(e.target.value)} placeholder="127.0.0.1:8777" />
            <label>TLS 端口</label>
            <input value={tlsListen} onChange={(e) => setTlsListen(e.target.value)} placeholder="127.0.0.1:9443" />
          </div>
          <div style={{ display: "flex", gap: 8, marginTop: 10, flexWrap: "wrap" }}>
            <button onClick={onStart}>启动</button>
            <button onClick={onApplyHosts} className="secondary">接管 hosts</button>
            <button onClick={onRevertHosts} className="secondary">释放 hosts</button>
          </div>
          <div style={{ marginTop: 10, display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
            <span>服务状态:</span>
            <span style={{ padding: "2px 8px", borderRadius: 12, background: svcOn ? "#dcfce7" : "#fee2e2", color: svcOn ? "#166534" : "#991b1b" }}>{svcOn ? "运行中" : "未启动"}</span>
            <span>hosts 状态:</span>
            <span style={{ padding: "2px 8px", borderRadius: 12, background: hostsStatus === "on" ? "#e0e7ff" : hostsStatus === "off" ? "#f3f4f6" : "#fef3c7", color: hostsStatus === "on" ? "#3730a3" : hostsStatus === "off" ? "#374151" : "#92400e" }}>{hostsStatus === "on" ? "已接管" : hostsStatus === "off" ? "已释放" : "未知"}</span>
          </div>
        </div>

        <div style={{ border: "1px solid #e5e7eb", borderRadius: 8, padding: 12, flex: 1, minWidth: 320 }}>
          <div style={{ fontWeight: 600, marginBottom: 8 }}>HTTPS 解密证书</div>
          <div style={{ display: "grid", gridTemplateColumns: "160px 1fr", gap: "8px 12px", alignItems: "center" }}>
            <label>Root CA</label>
            <input value={caPath} readOnly placeholder="点击获取路径" />
          </div>
          <div style={{ display: "flex", gap: 8, marginTop: 10, alignItems: "center", flexWrap: "wrap" }}>
            <button onClick={onExportCA} className="secondary">导出路径</button>
            <small style={{ color: "#6b7280" }}>请将根证书导入系统信任</small>
          </div>
        </div>
      </div>

      <div style={{ display: "flex", gap: 16, marginTop: 16, flexWrap: "wrap" }}>
        <div style={{ border: "1px solid #e5e7eb", borderRadius: 8, padding: 12, flex: 2, minWidth: 420 }}>
          <div style={{ fontWeight: 600, marginBottom: 8 }}>路由转发规则</div>
          <div style={{ display: "grid", gridTemplateColumns: "160px 1fr", gap: "8px 12px", alignItems: "center" }}>
            <label>OpenAI 上游</label>
            <input value={openai} onChange={(e) => setOpenai(e.target.value)} placeholder="https://api.openai.com" />
            <label>Anthropic 上游</label>
            <input value={anthropic} onChange={(e) => setAnthropic(e.target.value)} placeholder="https://api.anthropic.com" />
            <label>软件路径</label>
            <input value={appPath} onChange={(e) => setAppPath(e.target.value)} placeholder="/Applications/YourApp.app" />
          </div>
          <div style={{ display: "flex", gap: 8, marginTop: 10, flexWrap: "wrap" }}>
            <button onClick={onLaunch}>拉起</button>
            <button onClick={onSave} className="secondary">保存配置</button>
            <button onClick={onClear} className="secondary">清理数据</button>
          </div>
        </div>
        <div style={{ border: "1px solid #e5e7eb", borderRadius: 8, padding: 12, flex: 1, minWidth: 320 }}>
          <div style={{ fontWeight: 600, marginBottom: 8 }}>日志</div>
          <div style={{ whiteSpace: "pre-wrap", background: "#0b1020", color: "#e5e7eb", padding: 8, borderRadius: 6, height: 200, overflow: "auto" }}>
            {logs.map((l, i) => (
              <div key={i}>{l}</div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
