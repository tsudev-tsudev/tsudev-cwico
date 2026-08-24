/**
 * Bilingual strings.
 *
 * Vietnamese is the default because that is the primary audience; English is
 * kept complete because this ships to the Microsoft Store. A flat key space
 * with a typed `TranslationKey` means a missing translation is a compile
 * error rather than a `[object Object]` in a dialog.
 */

import type { Locale, SafetyClass, SourceKind } from "./types";

const vi = {
  "app.title": "tsudev-cwico",
  "app.subtitle": "Dọn dẹp & gỡ phần mềm Windows chuyên sâu",

  "nav.software": "Phần mềm",
  "nav.tweaks": "Tinh chỉnh",
  "nav.activity": "Nhật ký",
  "nav.about": "Giới thiệu",

  "scan.run": "Quét hệ thống",
  "scan.rerun": "Quét lại",
  "scan.running": "Đang quét…",
  "scan.quick": "Quét nhanh",
  "scan.full": "Quét đầy đủ",
  "scan.deep": "Quét sâu",
  "scan.deepHint":
    "Bao gồm cả tàn dư, thành phần hệ thống và đo dung lượng đĩa. Chậm hơn đáng kể.",
  "scan.empty": "Chưa quét. Nhấn “Quét hệ thống” để bắt đầu.",
  "scan.noResults": "Không có mục nào khớp bộ lọc hiện tại.",
  "scan.itemsFound": "mục",
  "scan.reclaimable": "Có thể thu hồi",
  "scan.lastScan": "Lần quét gần nhất",

  "filter.search": "Tìm theo tên, nhà phát hành, mã gói…",
  "filter.allKinds": "Tất cả loại",
  "filter.allSafety": "Tất cả mức an toàn",
  "filter.sort": "Sắp xếp",
  "sort.name": "Tên",
  "sort.size": "Dung lượng",
  "sort.safety": "Mức an toàn",
  "sort.publisher": "Nhà phát hành",
  "sort.kind": "Loại",
  "sort.installDate": "Ngày cài",

  "safety.safe": "An toàn",
  "safety.caution": "Cẩn trọng",
  "safety.unknown": "Chưa rõ",
  "safety.critical": "Trọng yếu",
  "safety.safe.short": "Gỡ được, Windows không ảnh hưởng",
  "safety.caution.short": "Gỡ được nhưng mất một tính năng phụ",
  "safety.unknown.short": "Chưa phân loại - hãy tự kiểm tra",
  "safety.critical.short": "Không thể gỡ - hệ điều hành phụ thuộc",

  "kind.registry_uninstall": "Chương trình",
  "kind.appx_package": "Ứng dụng UWP",
  "kind.appx_provisioned": "Gói nạp sẵn",
  "kind.windows_service": "Dịch vụ",
  "kind.scheduled_task": "Tác vụ lập lịch",
  "kind.startup_entry": "Khởi động cùng máy",
  "kind.windows_capability": "Thành phần Windows",
  "kind.optional_feature": "Tính năng tùy chọn",
  "kind.leftover": "Tàn dư",

  "select.all": "Chọn tất cả mục An toàn",
  "select.none": "Bỏ chọn tất cả",
  "select.count": "đã chọn",
  "select.blocked": "Mục Trọng yếu không thể chọn",

  "action.review": "Xem kế hoạch",
  "action.preview": "Chạy thử (không thay đổi gì)",
  "action.remove": "Gỡ bỏ",
  "action.cancel": "Hủy",
  "action.close": "Đóng",
  "action.back": "Quay lại",
  "action.confirm": "Tôi hiểu, tiếp tục",
  "action.details": "Chi tiết",
  "action.copy": "Sao chép",
  "action.openBackups": "Mở thư mục sao lưu",

  "plan.title": "Kế hoạch gỡ bỏ",
  "plan.steps": "bước",
  "plan.rejected": "Bị bỏ qua vì lý do an toàn",
  "plan.deepClean": "Xóa tận gốc tàn dư (thư mục + Registry)",
  "plan.restorePoint": "Tạo điểm khôi phục hệ thống trước khi gỡ",
  "plan.registryBackup": "Xuất sao lưu .reg cho các khóa sẽ can thiệp",
  "plan.killProcesses": "Tắt tiến trình đang chạy của phần mềm",
  "plan.dryRun": "Chỉ chạy thử - không thay đổi hệ thống",
  "plan.empty": "Không còn mục nào sau khi kiểm tra an toàn.",
  "plan.confirmTitle": "Xác nhận thao tác không thể hoàn tác dễ dàng",
  "plan.confirmBody":
    "Bạn sắp gỡ bỏ phần mềm khỏi máy này. Điểm khôi phục hệ thống và bản sao lưu .reg sẽ được tạo trước, nhưng hãy chắc chắn bạn thực sự không cần những mục dưới đây.",

  "confirm.caution.title": "Mục này cần xác nhận riêng",
  "confirm.caution.body":
    "Mục này được phân loại Cẩn trọng hoặc Chưa rõ. Gỡ bỏ sẽ thành công, nhưng bạn có thể mất một tính năng.",

  "run.title": "Đang thực hiện",
  "run.done": "Hoàn tất",
  "run.succeeded": "thành công",
  "run.failed": "thất bại",
  "run.skipped": "bỏ qua",
  "run.freed": "đã giải phóng",
  "run.restorePointCreated": "Đã tạo điểm khôi phục",
  "run.rebootRequired": "Cần khởi động lại máy để hoàn tất.",
  "run.transactionLog": "Nhật ký giao dịch",

  "tweaks.title": "Tinh chỉnh hệ thống",
  "tweaks.subtitle":
    "Từng thay đổi đều có thể bật/tắt riêng và hầu hết đều hoàn tác được.",
  "tweaks.recommended": "Chọn bộ khuyến nghị",
  "tweaks.apply": "Áp dụng đã chọn",
  "tweaks.revert": "Hoàn tác đã chọn",
  "tweaks.oneWay": "Không thể hoàn tác",
  "tweaks.requiresRestart": "Cần khởi động lại",

  "activity.title": "Nhật ký hoạt động",
  "activity.empty": "Chưa có hoạt động nào trong phiên này.",

  "about.title": "Giới thiệu",
  "about.engine": "Bộ máy",
  "about.safetyDb": "Cơ sở dữ liệu an toàn",
  "about.rules": "quy tắc",
  "about.backupDir": "Thư mục sao lưu",
  "about.os": "Hệ điều hành",
  "about.website": "Truy cập tsudev.com",

  "elevation.required": "Chưa có quyền Administrator",
  "elevation.body":
    "Quét vẫn chạy được nhưng danh sách sẽ thiếu, và không thể gỡ bất cứ thứ gì. Hãy khởi động lại với quyền quản trị.",
  "elevation.relaunch": "Khởi động lại với quyền Administrator",
  "elevation.ok": "Đang chạy với quyền Administrator",

  "restore.unavailable": "System Protection đang tắt",
  "restore.unavailableBody":
    "Không thể tạo điểm khôi phục. Hãy bật System Protection cho ổ hệ thống trong Settings › System › About › System protection trước khi gỡ bất cứ thứ gì.",

  "detail.publisher": "Nhà phát hành",
  "detail.version": "Phiên bản",
  "detail.size": "Dung lượng",
  "detail.installed": "Ngày cài",
  "detail.location": "Vị trí cài đặt",
  "detail.registryKey": "Khóa Registry",
  "detail.uninstallString": "Lệnh gỡ",
  "detail.quietUninstall": "Lệnh gỡ im lặng",
  "detail.package": "Tên gói",
  "detail.service": "Tên dịch vụ",
  "detail.processes": "Tiến trình",
  "detail.running": "Đang chạy",
  "detail.why": "Vì sao được phân loại như vậy",

  "error.title": "Đã xảy ra lỗi",
  "update.gate.title": "Có phiên bản mới",
  "update.gate.lead":
    "Cần cập nhật lên phiên bản mới nhất trước khi tiếp tục sử dụng.",
  "update.gate.why":
    "Cơ sở dữ liệu an toàn quyết định phần mềm nào được phép gỡ. Khi một quy tắc được sửa - ví dụ một mục từng bị xếp nhầm là An toàn - bản sửa đi kèm phiên bản mới. Chạy bản cũ nghĩa là đang dùng đánh giá an toàn đã lỗi thời cho chính máy của bạn.",
  "update.current": "Đang dùng",
  "update.new": "Phiên bản mới",
  "update.published": "Phát hành",
  "update.notes": "Có gì thay đổi",
  "update.button": "Cập nhật",
  "update.downloading": "Đang tải…",
  "update.installing": "Đang cài đặt…",
  "update.restarting": "Sắp khởi động lại…",
  "update.failed": "Cập nhật không thành công",
  "update.retry": "Thử lại",
  "update.checking": "Đang kiểm tra cập nhật…",
  "update.checkFailed": "Chưa kiểm tra được cập nhật",
  "update.checkFailedBody":
    "Không kết nối được máy chủ cập nhật, phần mềm vẫn dùng bình thường. Sẽ kiểm tra lại ở lần khởi động sau.",
  "update.upToDate": "Đang dùng phiên bản mới nhất",
  "theme.toggle": "Đổi giao diện sáng/tối",
  "locale.toggle": "Đổi ngôn ngữ",
} as const;

export type TranslationKey = keyof typeof vi;

const en: Record<TranslationKey, string> = {
  "app.title": "tsudev-cwico",
  "app.subtitle": "Deep Windows debloater & software removal",

  "nav.software": "Software",
  "nav.tweaks": "Tweaks",
  "nav.activity": "Activity",
  "nav.about": "About",

  "scan.run": "Scan system",
  "scan.rerun": "Rescan",
  "scan.running": "Scanning…",
  "scan.quick": "Quick scan",
  "scan.full": "Full scan",
  "scan.deep": "Deep scan",
  "scan.deepHint":
    "Includes residue, system components and disk measurement. Noticeably slower.",
  "scan.empty": "Nothing scanned yet. Press “Scan system” to begin.",
  "scan.noResults": "No items match the current filters.",
  "scan.itemsFound": "items",
  "scan.reclaimable": "Reclaimable",
  "scan.lastScan": "Last scan",

  "filter.search": "Search by name, publisher, package id…",
  "filter.allKinds": "All kinds",
  "filter.allSafety": "All safety classes",
  "filter.sort": "Sort",
  "sort.name": "Name",
  "sort.size": "Size",
  "sort.safety": "Safety",
  "sort.publisher": "Publisher",
  "sort.kind": "Kind",
  "sort.installDate": "Installed",

  "safety.safe": "Safe",
  "safety.caution": "Caution",
  "safety.unknown": "Unknown",
  "safety.critical": "Critical",
  "safety.safe.short": "Removable with no effect on Windows",
  "safety.caution.short": "Removable, but costs a secondary feature",
  "safety.unknown.short": "Unclassified - check it yourself",
  "safety.critical.short": "Cannot be removed - Windows depends on it",

  "kind.registry_uninstall": "Program",
  "kind.appx_package": "UWP app",
  "kind.appx_provisioned": "Provisioned package",
  "kind.windows_service": "Service",
  "kind.scheduled_task": "Scheduled task",
  "kind.startup_entry": "Startup entry",
  "kind.windows_capability": "Windows capability",
  "kind.optional_feature": "Optional feature",
  "kind.leftover": "Leftover",

  "select.all": "Select all Safe items",
  "select.none": "Clear selection",
  "select.count": "selected",
  "select.blocked": "Critical items cannot be selected",

  "action.review": "Review plan",
  "action.preview": "Dry run (changes nothing)",
  "action.remove": "Remove",
  "action.cancel": "Cancel",
  "action.close": "Close",
  "action.back": "Back",
  "action.confirm": "I understand, continue",
  "action.details": "Details",
  "action.copy": "Copy",
  "action.openBackups": "Open backup folder",

  "plan.title": "Removal plan",
  "plan.steps": "steps",
  "plan.rejected": "Skipped for safety",
  "plan.deepClean": "Deep clean residue (folders + registry)",
  "plan.restorePoint": "Create a System Restore Point first",
  "plan.registryBackup": "Export .reg backups of the keys being touched",
  "plan.killProcesses": "Terminate the software's running processes",
  "plan.dryRun": "Dry run - nothing is changed",
  "plan.empty": "Nothing left to do after the safety checks.",
  "plan.confirmTitle": "Confirm an action that is not easily undone",
  "plan.confirmBody":
    "You are about to remove software from this machine. A System Restore Point and .reg backups are taken first, but make sure you genuinely do not need the items below.",

  "confirm.caution.title": "This item needs its own confirmation",
  "confirm.caution.body":
    "This item is classified Caution or Unknown. Removal will succeed, but you may lose a feature.",

  "run.title": "Running",
  "run.done": "Finished",
  "run.succeeded": "succeeded",
  "run.failed": "failed",
  "run.skipped": "skipped",
  "run.freed": "freed",
  "run.restorePointCreated": "Restore point created",
  "run.rebootRequired": "A restart is required to finish.",
  "run.transactionLog": "Transaction log",

  "tweaks.title": "System tweaks",
  "tweaks.subtitle":
    "Every change is individually selectable, and most of them can be undone.",
  "tweaks.recommended": "Select the recommended set",
  "tweaks.apply": "Apply selected",
  "tweaks.revert": "Revert selected",
  "tweaks.oneWay": "Cannot be undone",
  "tweaks.requiresRestart": "Needs a restart",

  "activity.title": "Activity log",
  "activity.empty": "Nothing has happened in this session yet.",

  "about.title": "About",
  "about.engine": "Engine",
  "about.safetyDb": "Safety database",
  "about.rules": "rules",
  "about.backupDir": "Backup folder",
  "about.os": "Operating system",
  "about.website": "Visit tsudev.com",

  "elevation.required": "Not running as Administrator",
  "elevation.body":
    "Scanning still works but the list will be incomplete, and nothing can be removed. Relaunch with administrative rights.",
  "elevation.relaunch": "Relaunch as Administrator",
  "elevation.ok": "Running as Administrator",

  "restore.unavailable": "System Protection is off",
  "restore.unavailableBody":
    "No restore point can be created. Turn on System Protection for the system drive in Settings › System › About › System protection before removing anything.",

  "detail.publisher": "Publisher",
  "detail.version": "Version",
  "detail.size": "Size",
  "detail.installed": "Installed",
  "detail.location": "Install location",
  "detail.registryKey": "Registry key",
  "detail.uninstallString": "Uninstall command",
  "detail.quietUninstall": "Silent uninstall command",
  "detail.package": "Package name",
  "detail.service": "Service name",
  "detail.processes": "Processes",
  "detail.running": "Running",
  "detail.why": "Why it is classified this way",

  "error.title": "Something went wrong",
  "update.gate.title": "A new version is available",
  "update.gate.lead":
    "This update must be installed before you can continue.",
  "update.gate.why":
    "The safety database decides what this tool will and will not remove. When a rule is corrected - something classified Safe that should not have been, say - the fix ships as a new version. Running an old build means running an out-of-date idea of what is safe to delete on your machine.",
  "update.current": "Installed",
  "update.new": "New version",
  "update.published": "Published",
  "update.notes": "What changed",
  "update.button": "Update",
  "update.downloading": "Downloading…",
  "update.installing": "Installing…",
  "update.restarting": "Restarting…",
  "update.failed": "The update did not complete",
  "update.retry": "Try again",
  "update.checking": "Checking for updates…",
  "update.checkFailed": "Could not check for updates",
  "update.checkFailedBody":
    "The update server could not be reached, so the app started normally. It will check again next time.",
  "update.upToDate": "You are on the latest version",
  "theme.toggle": "Toggle light/dark theme",
  "locale.toggle": "Change language",
};

const TABLES: Record<Locale, Record<TranslationKey, string>> = { vi, en };

export function translator(locale: Locale) {
  const table = TABLES[locale];
  return (key: TranslationKey): string => table[key];
}

export function safetyLabel(locale: Locale, safety: SafetyClass): string {
  return translator(locale)(`safety.${safety}` as TranslationKey);
}

export function safetyBlurb(locale: Locale, safety: SafetyClass): string {
  return translator(locale)(`safety.${safety}.short` as TranslationKey);
}

export function kindLabel(locale: Locale, kind: SourceKind): string {
  return translator(locale)(`kind.${kind}` as TranslationKey);
}

/** Byte counts, in the reader's locale conventions. */
export function formatBytes(locale: Locale, bytes: number | undefined): string {
  if (bytes === undefined || bytes === 0) return "-";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit + 1 < units.length) {
    value /= 1024;
    unit += 1;
  }
  const formatted = new Intl.NumberFormat(locale === "vi" ? "vi-VN" : "en-US", {
    maximumFractionDigits: unit === 0 ? 0 : 1,
  }).format(value);
  return `${formatted} ${units[unit]}`;
}

export function formatDateTime(locale: Locale, iso: string | undefined): string {
  if (!iso) return "-";
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return new Intl.DateTimeFormat(locale === "vi" ? "vi-VN" : "en-US", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}
