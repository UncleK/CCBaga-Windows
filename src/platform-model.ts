/** Shared Windows language/timezone aliases for measured environment checks. */
import zones from '../src-tauri/src/windows/timezones.json';
const windowsZones: Record<string,string> = zones.iana;
export function sameTimezone(a:string,b:string,platform:string):boolean {
  return a===b || (platform==='windows' && !!windowsZones[a] && windowsZones[a]===windowsZones[b]);
}
export function languageTag(value:string):string {
  const tag=value.replaceAll('_','-');
  if (['zh-CN','zh-SG'].includes(tag)) return 'zh-Hans';
  if (['zh-TW','zh-HK','zh-MO'].includes(tag)) return 'zh-Hant';
  return tag;
}
export function sameLanguage(a:string,b:string):boolean {
  const first=languageTag(a),second=languageTag(b);
  return !!first && !!second && (first===second || first.startsWith(second+'-') || second.startsWith(first+'-'));
}
export function samePlatform(observed:string,platform:string):boolean {
  const token={windows:'win',linux:'linux',macos:'mac'}[platform];
  return !!token && observed.toLowerCase().includes(token);
}
