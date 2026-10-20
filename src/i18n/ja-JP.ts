import type { RawDictionary } from ".";

export const dict: RawDictionary = {
  common: {
    loading: "読み込み中",
    operationFailed: "操作に失敗しました",
  },

  app: {
    mode: {
      fixed: "固定",
      random: "ランダム",
    },
    engine: {
      notRunning: "エンジン停止中",
      running: "実行中 · {{ mode }} · 現在",
      start: "エンジンを起動",
      stop: "エンジンを停止",
      confirmStop: "停止しますか？",
      lastError: "最新のエラー：",
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
    selectAll: "すべて選択",
    clearAll: "選択を解除",
    individual: "個別設定",
    unified: "すべてのモニターを統一",
    unifiedOn:
      "1つのテーマを全モニターに適用します。下の個別設定はロックされます。",
    unifiedOff: "オフにするとモニターごとにテーマを指定できます。",
    randomEmpty:
      "候補プールが空です。エンジンが選べる壁紙がありません。少なくとも1つ残してください。",
    randomDirty: "候補プール {{ n }} 件 · {{ excluded }} 件除外 · 未保存",
    randomNormal:
      "候補プール {{ n }} 件 · チェックを外すと毎日のシャッフルから除外",
  },

  stage: {
    notInstalled: "このテーマは未インストールか壁紙がありません",
    download: "ダウンロード",
    downloading: "ダウンロード中…",
    cancelDownload: "キャンセル",
    downloadFailed: "ダウンロードに失敗しました：",
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
      today: "今日のテーマ",
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
    loadFailed: "設定を読み込めませんでした",
    retry: "再試行",
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
      titleBar: "タイトルバーを Windows のテーマ色にする",
      titleBarDesc:
        "オフのときは Dwall の背景色でタイトルバーを描画し、ウィンドウと一体化します。色まで合わせられるのは Windows 11 のみで、Windows 10 はライト/ダークの同期までです。",
      contentBorder: "タイトルバーと内容を区切る",
      contentBorderDesc:
        "内容領域の上端に境界線を引き、タイトルバーがウィンドウと同じ色でも見つけやすく、ドラッグしやすくします。",
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
      colorScheme: "システムのライト/ダークを自動で切り替える",
      colorSchemeDesc:
        "時刻ではなく太陽の位置に従います。日の出後はライト、日の入り後はダーク（市民薄明 -6°）で、切り替え付近には遊びがあり頻繁には反転しません。",
    },

    paths: {
      title: "ディレクトリとダウンロード",
      themesDir: "テーマディレクトリ",
      themesDirDesc:
        "ローカルテーマとサムネイルの保存場所。変更すると既存テーマが移動します。",
      selectDir: "ディレクトリを選択",
      openDir: "フォルダを開く",
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
      changelog: "変更履歴",
      changelogSub: "リリースノート",
      whatsNewFor: "{{ version }} の更新内容",
      whatsNew: "更新内容",
      sourceSub: "github.com",
      logDir: "ログディレクトリ",
      logDirSub: "フォルダーを開く",
      checkUpdate: "アップデートを確認",
      checking: "確認中…",
      updateToastTitle: "新しいバージョン {{ version }} があります",
      updateToastHint:
        "「このアプリについて」からダウンロードしてインストールできます。",
      updateToastAction: "開く",
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
