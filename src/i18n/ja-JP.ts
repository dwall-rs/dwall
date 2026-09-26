import type { RawDictionary } from ".";

export const dict: RawDictionary = {
  common: {
    loading: "読み込み中",
  },

  app: {
    mode: {
      fixed: "固定",
      random: "ランダム",
    },
    engine: {
      notRunning: "エンジン停止中",
      running: "実行中 · 現在",
      daily: "実行中 · 毎日シャッフル",
      start: "エンジンを起動",
      stop: "エンジンを停止",
      confirmStop: "停止しますか？",
      altitude: "高度",
      azimuth: "方位",
    },
    appearance: {
      light: "ライト",
      dark: "ダーク",
      system: "システム",
      title: "外観",
    },
  },

  library: {
    title: "テーマライブラリ",
    toggleOpen: "テーマライブラリを閉じる",
    toggleClosed: "テーマライブラリを開く",
    hintFixed: "クリックして{{ name }}に割り当て",
    hintRandom: "クリックしてプールに追加/削除",
    search: "テーマを検索…",
  },

  scope: {
    monitors: "スコープ · モニター",
    allMonitors: "すべてのモニター",
    pool: "候補プール",
    selectedCount: "選択中 {{ n }}/{{ total }}",
    individual: "個別設定",
    unified: "すべてのモニターを統一",
    unifiedOn:
      "1つのテーマを全モニターに適用します。下の個別設定はロックされます。",
    unifiedOff: "オフにするとモニターごとにテーマを指定できます。",
    searchThemes: "テーマを検索…",
    randomEmpty:
      "候補プールが空です。エンジンが選べる壁紙がありません。少なくとも1つ残してください。",
    randomDirty: "候補プール {{ n }} 件 · {{ excluded }} 件除外 · 未保存",
    randomNormal:
      "候補プール {{ n }} 件 · チェックを外すと毎日のシャッフルから除外",
  },

  stage: {
    notInstalled: "このテーマは未インストールか壁紙がありません",
    applyTo: "このテーマを{{ name }}に書き込み",
    stop: "停止",
    apply: "このテーマを適用",
    altitude: "高度",
    azimuth: "方位",
    matched: "現在一致",
    match: "一致",
    stripTitle: "収録壁紙 · 太陽位置で一致",
    stripHint: "クリックしてその太陽位置の壁紙をプレビュー",
    sunPath: "太陽の軌道",
    horizon: "0° 地平線",
    east: "東·90°",
    south: "南·180°",
    west: "西·270°",
    randomIntro:
      "毎日候補プールから1つのテーマをシャッフルして全モニターに適用します。エディターはプールを選ぶだけで、どのテーマが選ばれるかは実行時にエンジンが決定します。",
    stats: {
      pool: "候補プール / 件",
      range: "範囲",
      all: "すべて",
      exclude: "{{ n }} 件除外",
      target: "対象モニター",
      period: "切替周期",
      targetValue: "すべて",
      periodValue: "24 時間",
    },
    collageTitle: "候補プールのコラージュ",
    collageNote:
      "{{ n }} 件が毎日シャッフルに参加 · プールのプレビューのみ、抽選結果ではありません",
    emptyTitle: "候補プールが空です",
    emptyDesc:
      "すべてのテーマを除外しました。毎日のシャッフル用に少なくとも1つ残してください。",
  },

  commit: {
    applied: "適用済み {{ theme }}",
    notApplied: "未適用",
    unsaved: "未保存の変更",
    saved: "保存済み",
    savedAt: "保存済み {{ time }}",
    discard: "破棄",
    saving: "保存中…",
    save: "設定を保存",
  },

  settings: {
    title: "設定",
    subtitle: "preferences",
    unsaved: "未保存",
    saveFailed: "保存に失敗",
    discard: "破棄",
    save: "設定を保存",
    saving: "保存中…",
    intro:
      "設定はエンジンの太陽角マッチング動作を制御します。トグルは即時保存され、保存アイコン付きの項目は手動確認が必要です。",

    appearance: {
      title: "外観と言語",
      appearanceLabel: "外観",
      appearanceDesc:
        "ライト、ダーク、またはシステムに従う。システムに従うと自動で切り替わります。",
      languageLabel: "言語",
      languageDesc: "UIの言語。切り替えると即時反映されます。",
    },

    engine: {
      title: "エンジンと太陽角マッチング",
      launchAtStartup: "スタートアップで起動",
      launchAtStartupDesc:
        "バックグラウンドのエンジンのみを起動し、ウィンドウを開かず、メモリをほとんど消費しません。",
      interval: "チェック間隔",
      secondsUnit: "秒",
      intervalDesc:
        "エンジンは N 秒ごとに太陽高度を再計算し、その時刻の壁紙を選びます。",
      autoCoords: "座標を自動取得",
      autoCoordsDesc:
        "緯度経度は太陽高度の計算に使われます。オフにすると手動で入力できます。",
      manualCoords: "手動座標",
      manualCoordsDesc: "緯度 -90~90、経度 -180~180、高度（m）。",
      latitude: "緯度",
      longitude: "経度",
      altitude: "高度",
      save: "保存",
      lockScreen: "ロック画面の壁紙も設定",
      lockScreenDesc: "ロック画面を変更したくない場合はオフにしてください。",
    },

    paths: {
      title: "ディレクトリとダウンロード",
      themesDir: "テーマディレクトリ",
      themesDirDesc:
        "ローカルテーマとサムネイルの保存場所。変更すると既存テーマが移動します。",
      selectDir: "ディレクトリを選択",
      network: "ネットワーク",
      networkDesc:
        "テーマのダウンロードやサムネイルの読み込みには GitHub ミラーテンプレートや SOCKS5 プロキシが必要な場合があります。",
      none: "オフ",
      mirror: "ミラー",
      socks5: "SOCKS5",
      mirrorTemplate: "GitHub ミラーテンプレート",
      mirrorDesc:
        "ミラーテンプレートはダウンロードを高速化します。一部地域では GitHub が制限されています。利用可能なテンプレート:",
      viewTemplates: "テンプレート一覧を見る ↗",
      address: "ホスト",
      port: "ポート",
      save: "保存",
    },

    about: {
      title: "このアプリについて",
      tagline: "Dwall · 太陽位置駆動",
      logoAlt: "Dwall ロゴ",
      sourceCode: "ソースコード",
      sourceSub: "github.com",
      logDir: "ログディレクトリ",
      logDirSub: "フォルダーを開く",
      checkUpdate: "アップデートを確認",
      checking: "確認中…",
      upToDate: "最新版です",
      available: "{{ version }} に更新できます",
      download: "ダウンロードしてインストール",
      downloading: "ダウンロード中…",
      ready: "再起動して完了",
      failed: "更新の確認に失敗しました",
      retry: "再試行",
    },
  },
};
