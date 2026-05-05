import type { RawDictionary } from ".";

export const dict: RawDictionary = {
  common: {
    message: {
      githubStar:
        "このアプリがお役に立ちましたら、GitHubでこのプロジェクトにStarをお願いします：",

      locationPermission:
        "位置情報の許可が有効になっていません。手動で位置情報を有効にするか、手動で座標を設定してください。\n\n手動で座標を設定しますか？\n「はい」をクリックすると手動で座標を設定し、「いいえ」をクリックすると位置情報を有効にします。",

      updateAvailable:
        "新しいバージョン {{ version }} が検出されました。現在のバージョンは {{ currentVersion }} です。左下隅のアップグレードボタンをクリックしてダウンロードとインストールを行ってください。",
    },
  },

  settings: {
    unit: {
      hour: "時間",
      second: "秒",
    },

    button: {
      openLogDirectory: "ログディレクトリを開く",
      selectDirectory: "ディレクトリを選択",
    },

    label: {
      automaticallyRetrieveCoordinates: "座標を自動取得",
      automaticallySwitchModes: "ダークモードまたはライトモードに自動切替",
      checkInterval: "チェック間隔",
      githubMirrorTemplate: "Github ミラーテンプレート",
      launchAtStartup: "起動時に起動",
      setLockScreenWallpaperSimultaneously: "ロック画面の壁紙も同時に設定",
      sourceCode: "ソースコード",
      themesDirectory: "テーマディレクトリ",
      version: "バージョン",
      language: "言語",
    },

    help: {
      automaticallySwitchModes:
        "ライト/ダークモードを自動的に切り替えたくない場合は、このオプションを無効にしてください。",
      githubMirror:
        "Githubミラーテンプレートはダウンロードを高速化するために使用されます。国や地域によってはネットワーク制限のためGithubへのアクセスに失敗し、ダウンロードが失敗することがあります。サムネイルを正しく読み込み、テーマをダウンロードするには、Githubミラーテンプレートを設定する必要があります。このボタンをクリックすると、利用可能なGithubミラーテンプレートを表示できます：",
      launchAtStartup:
        "自動起動はバックグラウンドプロセスのみを起動し、グラフィカルプログラムは起動しません。また、メモリをあまり消費しません。",
      manuallySetCoordinates:
        "手動で座標を設定する際は、WGS84座標系（国際標準、中国のユーザーは注意してください）を使用する必要があります。そうしないと、座標オフセットの問題が発生し、壁紙の位置合わせが不正確になる可能性があります。",
      setLockScreenWallpaperSimultaneously:
        "ロック画面の壁紙も同時に設定したくない場合は、このオプションを無効にしてください。",
      updatedFailed:
        "ホットアップデートを完了できませんでした。このメッセージの後ろにあるダウンロードボタンをクリックして、新しいバージョンを手動でダウンロードしてください：",
    },

    placeholder: {
      latitude: "緯度を入力",
      longitude: "経度を入力",
    },

    tooltip: {
      openThemesDirectory: "クリックしてテーマディレクトリを開きます。",
      checkForNewVersion: "クリックして新バージョンを確認",
    },

    message: {
      changeThemesDirectory:
        "テーマディレクトリを {{ directory }} に変更しますか？",
      checkIntervalUpdated: "チェック間隔が {{ interval }} 秒に更新されました",
      disableStartupFailed:
        "起動時の自動起動の無効化に失敗しました：\n{{ error }}",
      githubMirrorTemplateUpdated:
        "Githubミラーテンプレートが {{ template }} に更新されました",
      invalidNumber: "有効な数値を入力してください。",
      manualCoordinatesSaved:
        "座標が保存されました。次に適用するテーマを選択できます。",
      numberTooLarge: "{{ max }} より小さい数値を入力してください。",
      numberTooSmall: "{{ min }} より大きい数値を入力してください。",
      SaveManualCoordinatesFailed: "座標の保存に失敗しました：\n{{ error }}",
      startupFailed: "起動時の自動起動の有効化に失敗しました：\n{{ error }}",
      switchAutoModesFailed:
        "自動ライト/ダークモードの切り替えに失敗しました：\n{{ error }}",
      switchToManualCoordinatesFailed:
        "手動座標への切り替えに失敗しました：\n{{ error }}",
      movedThemesDirectory:
        "テーマディレクトリが {{ directory }} に移動されました",
      isLatestVersion: "最新バージョンを使用しています。",
    },
  },

  sidebar: {
    tooltip: {
      newVersionAvailable:
        "新しいバージョンが利用可能です！このボタンをクリックして更新してください。",
      settings: "設定",
    },
  },

  theme: {
    label: {
      selectMonitor: "モニターを選択",
    },

    button: {
      apply: "適用",
      cancel: "キャンセル",
      download: "ダウンロード",
      stop: "停止",
    },

    message: {
      applyThemeFailed: "テーマの適用に失敗しました：\n{{ error }}",
      downloadCancelled: "ダウンロードがキャンセルされました",
      downloadFailed: "テーマのダウンロードに失敗しました：\n{{ error }}",
      fileSizeWarning:
        "ファイルサイズを取得できないため、ダウンロードの進行状況を計算できません。レスポンスヘッダの転送をサポートするGithubミラーテンプレートに切り替えてください",
    },

    title: {
      downloadFailed: "ダウンロード失敗",
    },
  },

  update: {
    button: {
      install: "インストール",
    },
    title: {
      downloadingNewVersion: "新しいバージョン {{ version }} をダウンロード中",
    },
    message: {
      updateFailed: "更新に失敗しました：\n{{ error }}",
    },
  },
};
