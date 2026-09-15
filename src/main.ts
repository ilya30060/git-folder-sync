import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { appDataDir, join } from "@tauri-apps/api/path";

const $ = (id: string) => document.getElementById(id) as HTMLInputElement;
const status = document.getElementById("status")!;
const conflict = document.getElementById("conflict")!;

const isAndroid = /Android/i.test(navigator.userAgent);
let androidFolderUri = localStorage.getItem("androidFolderUri") || "";

function readConfig() {
  return {
    remote_url: $("url").value.trim(),
    username: $("username").value.trim(),
    token: $("token").value,
    branch: $("branch").value.trim() || "main",
    folder: $("folder").value.trim(),
  };
}

async function worktreePath() {
  const remote = $("url").value.trim();
  const bytes = new TextEncoder().encode(remote || "default");
  const hash = await crypto.subtle.digest("SHA-256", bytes);
  const id = [...new Uint8Array(hash)].map((b) => b.toString(16).padStart(2, "0")).join("");
  return join(await appDataDir(), "repositories", id);
}

function persistSettings() {
  localStorage.setItem("remote_url", $("url").value.trim());
  localStorage.setItem("username", $("username").value.trim());
  localStorage.setItem("branch", $("branch").value.trim() || "main");
  if (!isAndroid) localStorage.setItem("folder", $("folder").value.trim());
}

function restoreSettings() {
  $("url").value = localStorage.getItem("remote_url") || "";
  $("username").value = localStorage.getItem("username") || "";
  $("branch").value = localStorage.getItem("branch") || "main";
  if (isAndroid) {
    $("folder").value = androidFolderUri ? "Android SAF: выбранная папка" : "";
  } else {
    $("folder").value = localStorage.getItem("folder") || "";
  }
}

async function desktopConfig() {
  return { ...readConfig(), folder: isAndroid ? await worktreePath() : readConfig().folder };
}

async function androidImport() {
  if (!androidFolderUri) throw new Error("Сначала выберите папку Android.");
  const destination = await worktreePath();
  await invoke("plugin:android-saf|importTree", { uri: androidFolderUri, destination });
  return destination;
}

async function androidExport() {
  if (!androidFolderUri) throw new Error("Сначала выберите папку Android.");
  const source = await worktreePath();
  await invoke("plugin:android-saf|exportTree", { uri: androidFolderUri, source });
}

async function run(cmd: string) {
  status.textContent = "Выполняется…";
  conflict.hidden = true;
  try {
    persistSettings();
    const config = await desktopConfig();

    if (isAndroid && ["clone_or_initialize", "push_repo", "pull_repo", "repo_status"].includes(cmd)) {
      await androidImport();
    }

    const result = await invoke<string>(cmd, { config });

    if (isAndroid && cmd === "clone_or_initialize") {
      await androidExport();
    }
    if (isAndroid && cmd === "pull_repo") {
      await androidExport();
    }

    status.textContent = result;
    if (/разошлись|conflict/i.test(result)) conflict.hidden = false;
  } catch (e) {
    status.textContent = "Ошибка: " + String(e);
    if (/разошлись|conflict/i.test(String(e))) conflict.hidden = false;
  }
}

$("choose").onclick = async () => {
  try {
    if (isAndroid) {
      const result = await invoke<{ uri: string }>("plugin:android-saf|pickDirectory");
      const uri = result.uri;
      androidFolderUri = uri;
      localStorage.setItem("androidFolderUri", uri);
      $("folder").value = "Android SAF: выбранная папка";
    } else {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === "string") {
        $("folder").value = selected;
        localStorage.setItem("folder", selected);
      }
    }
    status.textContent = "Папка выбрана.";
  } catch (e) {
    if (String(e).toLowerCase().includes("cancelled")) return;
    status.textContent = "Ошибка выбора папки: " + String(e);
  }
};

$("saveCreds").onclick = () => run("save_credentials");
$("deleteCreds").onclick = () => run("delete_credentials");
$("clone").onclick = () => run("clone_or_initialize");
$("pull").onclick = () => run("pull_repo");
$("push").onclick = () => run("push_repo");
$("statusBtn").onclick = () => run("repo_status");
$("refreshStatus").onclick = () => run("repo_status");

restoreSettings();
