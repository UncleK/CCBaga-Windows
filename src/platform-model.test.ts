import {expect,test} from 'bun:test';
import {sameTimezone,sameLanguage,samePlatform} from './platform-model';
test('Windows regional timezone aliases match without accepting unrelated zones',()=>{
  expect(sameTimezone('America/Toronto','America/New_York','windows')).toBe(true);
  expect(sameTimezone('America/Toronto','America/New_York','macos')).toBe(false);
  expect(sameTimezone('Asia/Shanghai','Europe/Paris','windows')).toBe(false);
  expect(sameTimezone('missing-a','missing-b','windows')).toBe(false);
});
test('Windows language tags preserve script and region distinctions',()=>{
  expect(sameLanguage('zh-CN','zh-Hans')).toBe(true);
  expect(sameLanguage('zh-TW','zh-Hant')).toBe(true);
  expect(sameLanguage('zh-CN','zh-Hant')).toBe(false);
  expect(sameLanguage('en-US','en-GB')).toBe(false);
  expect(sameLanguage('en-US','en')).toBe(true);
  expect(sameLanguage('','')).toBe(false);
});
test('measured Windows platform is recognized independently of macOS',()=>{
  expect(samePlatform('Win32','windows')).toBe(true);
  expect(samePlatform('MacIntel','windows')).toBe(false);
  expect(samePlatform('Linux x86_64','linux')).toBe(true);
  expect(samePlatform('MacIntel','macos')).toBe(true);
});
