export type VerdictState = "HEALTHY" | "WARNING" | "CRITICAL_ISSUE" | "UNKNOWN";

export type EvidenceStrength = "CONFIRMED_FACT" | "STRONGLY_SUPPORTED" | "HYPOTHESIS" | "INCONCLUSIVE";

export type ServiceAvailability = "AVAILABLE" | "AUTH REQUIRED" | "DEGRADED" | "UNAVAILABLE" | "UNKNOWN";

export interface DiagnosticVerdict {
  overall_state: VerdictState;
  primary_issue_code: string;
  confidence_percent: number;
  evidence_strength: EvidenceStrength;
  confirmed_facts: string[];
  title_ru: string;
  explanation_ru: string;
  technical_evidence: string[];
  recommended_actions: string[];
  passed_checks_count: number;
  warning_checks_count: number;
  failed_checks_count: number;
}

export interface ErrorDetectiveResult {
  category: string;
  error_code: string;
  title_ru: string;
  meaning_ru: string;
  confirmed_facts: string[];
  likely_cause_ru: string;
  suggested_tests: string[];
  safe_fix_ru: string[];
  sanitized_input: string;
}

export interface InterfaceInfo {
  name: string;
  description: string;
  status: string;
  is_up: boolean;
  mac?: string;
  ipv4: string[];
  ipv6: string[];
  gateways: string[];
  dns_servers: string[];
  is_virtual: boolean;
  adapter_type: string;
}

export interface DualStackStatus {
  ipv4_working: boolean;
  ipv4_rtt_ms?: number;
  ipv6_working: boolean;
  ipv6_rtt_ms?: number;
  preferred_family: string;
}

export interface NetworkDiagnostics {
  interfaces: InterfaceInfo[];
  dual_stack: DualStackStatus;
  primary_gateway?: string;
  primary_interface?: string;
  default_dns: string[];
}

export interface ResolverResult {
  resolver_name: string;
  resolver_ip: string;
  success: boolean;
  rtt_ms?: number;
  resolved_ips: string[];
  error?: string;
}

export interface DnsComparisonItem {
  hostname: string;
  system_result: ResolverResult;
  cloudflare_result: ResolverResult;
  google_result: ResolverResult;
  quad9_result: ResolverResult;
  is_tampered_or_blocked: boolean;
  divergence_detected: boolean;
  diagnosis_note: string;
}

export interface DnsDiagnostics {
  system_dns_working: boolean;
  average_dns_latency_ms: number;
  items: DnsComparisonItem[];
  any_tampering: boolean;
}

export interface WinInetProxy {
  enabled: boolean;
  server?: string;
  pac_url?: string;
  override_exceptions?: string;
}

export interface WinHttpProxy {
  is_direct: boolean;
  proxy_server?: string;
  bypass_list?: string;
  raw_output: string;
}

export interface ProxyEnvVars {
  http_proxy?: string;
  https_proxy?: string;
  all_proxy?: string;
  no_proxy?: string;
  has_any: boolean;
}

export interface ProxyEndpointCheck {
  endpoint: string;
  is_listening: boolean;
  tcp_rtt_ms?: number;
  can_forward_https: boolean;
  http_status?: number;
  response_error?: string;
}

export interface ProxyDiagnostics {
  wininet: WinInetProxy;
  winhttp: WinHttpProxy;
  env_vars: ProxyEnvVars;
  active_proxies: ProxyEndpointCheck[];
  has_conflict: boolean;
  conflict_description?: string;
  recommendation?: string;
}

export interface VirtualAdapter {
  name: string;
  description: string;
  status: string;
  is_active: boolean;
  vpn_type: string;
  ip_addresses: string[];
}

export interface RouteItem {
  destination: string;
  netmask: string;
  gateway: string;
  interface_name: string;
  metric: number;
  is_default_gateway: boolean;
}

export interface VpnDiagnostics {
  active_virtual_adapters: VirtualAdapter[];
  default_routes: RouteItem[];
  vpn_tunnel_detected: boolean;
  has_routing_collision: boolean;
  analysis_summary: string;
}

export type ServiceStatusKind =
  | "Pass"
  | "AuthBoundary"
  | "RateLimited"
  | "DnsFailure"
  | "TcpFailure"
  | "TlsFailure"
  | "ProxyFailure"
  | "GeoBlocked"
  | "ServerFailure"
  | "Unknown";

export interface EndpointCheck {
  url: string;
  host: string;
  dns_ok: boolean;
  tcp_ok: boolean;
  tls_ok: boolean;
  http_status?: number;
  latency_ms: number;
  server_header?: string;
  status_kind: ServiceStatusKind;
  details: string;
}

export interface ServiceProfileResult {
  id: string;
  name: string;
  category: string;
  overall_status: "pass" | "warning" | "fail" | "unknown";
  status_kind: ServiceStatusKind;
  primary_endpoint: string;
  endpoints: EndpointCheck[];
  latency_ms: number;
  summary_ru: string;
  summary_en: string;
  recommendation_ru?: string;
}

export interface FullDiagnosticReport {
  id: string;
  timestamp: string;
  verdict: DiagnosticVerdict;
  network: NetworkDiagnostics;
  dns: DnsDiagnostics;
  proxy: ProxyDiagnostics;
  vpn: VpnDiagnostics;
  services: ServiceProfileResult[];
}

export interface HistoryRecordSummary {
  id: string;
  timestamp: string;
  verdict_state: VerdictState;
  primary_issue_code: string;
  title_ru: string;
  explanation_ru: string;
  passed_count: number;
  warning_count: number;
  failed_count: number;
}

export interface ProgressUpdate {
  stage: string;
  percent: number;
  message: string;
}

export interface ChannelProbeResult {
  channel_id: string;
  channel_name: string;
  channel_type: string;
  is_available: boolean;
  latency_ms: number;
  http_status?: number;
  status_text: string;
  details_ru: string;
}

export interface ConnectionMatrixReport {
  target_url: string;
  target_name: string;
  timestamp: string;
  results: ChannelProbeResult[];
  comparative_verdict_ru: string;
}

export interface IdeConfigAudit {
  ide_id: string;
  ide_name: string;
  is_installed: boolean;
  config_path?: string;
  proxy_configured?: string;
  strict_ssl?: boolean;
  status: string;
  diagnosis_ru: string;
  recommendation_ru?: string;
}

export interface McpServerAudit {
  name: string;
  transport_type: string;
  command_or_url: string;
  is_executable_found: boolean;
  status: string;
  details_ru: string;
}

export interface McpFileAudit {
  client_name: string;
  file_path: string;
  exists: boolean;
  is_valid_json: boolean;
  servers_count: number;
  servers: McpServerAudit[];
  recommendation_ru?: string;
}

export interface FixPreview {
  fix_id: string;
  title_ru: string;
  description_ru: string;
  command_preview: string;
  subsystem: string;
  is_destructive: boolean;
  requires_admin: boolean;
  rollback_preview: string;
}

export interface FixExecutionResult {
  success: boolean;
  snapshot_id: string;
  message_ru: string;
  stdout: string;
  stderr: string;
}

export interface FreeProviderInfo {
  id: string;
  name: string;
  category: string;
  description_ru: string;
  primary_endpoint: string;
  base_url: string;
  docs_url: string;
  is_free_tier: boolean;
  popular_models: string[];
  recommended_for: string;
  direct_rtt_ms?: number;
  http_status?: number;
  is_accessible: boolean;
  status_text: string;
  auth_note: string;
}

export interface PublicDohInfo {
  name: string;
  provider: string;
  doh_url: string;
  primary_ip: string;
  secondary_ip: string;
  description_ru: string;
  features: string[];
  is_tested_ok: boolean;
  rtt_ms?: number;
}

export interface FreeHubReport {
  providers: FreeProviderInfo[];
  doh_resolvers: PublicDohInfo[];
  timestamp: string;
}

export interface OneClickStatus {
  adapter_name: string;
  current_dns: string[];
  is_cloudflare_dns: boolean;
  is_google_dns: boolean;
  current_openai_base_url?: string;
  active_ai_preset?: string;
  has_snapshot: boolean;
  snapshot_timestamp?: string;
}

export interface TestedOption {
  name: string;
  endpoint: string;
  is_working: boolean;
  latency_ms: number;
  note: string;
}

export interface AutoTroubleshootReport {
  success: boolean;
  title_ru: string;
  explanation_ru: string;
  selected_ai_name: string;
  selected_ai_base_url: string;
  selected_dns_name: string;
  selected_dns_ip: string;
  tested_ai: TestedOption[];
  tested_dns: TestedOption[];
  timestamp: string;
}

export interface NrptRuleSummary {
  name: string;
  namespace: string[];
  name_servers: string[];
  comment: string;
}

export interface AntigravityDoctorReport {
  ide_installed: boolean;
  ide_path?: string;
  ide_version?: string;
  language_server_installed: boolean;
  language_server_path?: string;
  language_server_size_mb?: number;
  ag_dns_running: boolean;
  ag_dns_pid?: number;
  loopback_relay_ready: boolean;
  nrpt_active: boolean;
  nrpt_rules: NrptRuleSummary[];
  ipv4_preferred: boolean;
  cloudcode_direct_status: string;
  cloudcode_rtt_ms?: number;
  accounts_status: string;
  accounts_ip?: string;
  ag_ls_proxy_env?: string;
  overall_status_ru: string;
  status_badge: "OPTIMAL" | "WARNING" | "DISABLED";
}

export interface FlowUnlockInfo {
  flow_url: string;
  script_code: string;
  bookmarklet: string;
  is_window_open: boolean;
  extension_folder_path: string;
  default_browser_name: string;
}

export interface HostsEntryInfo {
  line_number: number;
  ip: string;
  domain: string;
  is_ai_domain: boolean;
  is_commented: boolean;
  is_suspicious_or_dead: boolean;
  note_ru: string;
}

export interface HostsDoctorReport {
  hosts_path: string;
  total_lines: number;
  ai_entries_count: number;
  suspicious_entries_count: number;
  entries: HostsEntryInfo[];
  has_backup: boolean;
  status_ru: string;
}

export interface CliProxyStatus {
  git_proxy?: string;
  npm_proxy?: string;
  env_http_proxy?: string;
  env_https_proxy?: string;
  is_any_configured: boolean;
  effective_summary_ru: string;
}

export interface BenchmarkTargetResult {
  id: string;
  name: string;
  category: string;
  endpoint: string;
  latency_ms?: number;
  http_status?: number;
  is_available: boolean;
  speed_rating: "ULTRA_FAST" | "FAST" | "NORMAL" | "SLOW" | "BLOCKED";
  note_ru: string;
}

export interface SpeedBenchmarkReport {
  timestamp: string;
  fastest_provider_name?: string;
  fastest_provider_rtt_ms?: number;
  total_tested: number;
  available_count: number;
  results: BenchmarkTargetResult[];
  recommendation_ru: string;
}
