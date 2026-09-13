import QtQuick
import QtQuick.Window
import SddmComponents 2.0

Rectangle {
    id: root
    width: Screen.width
    height: Screen.height
    color: "#181216"
    readonly property real s: height / 1080

    // Wayland Cursor Fix
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.ArrowCursor
        z: -1
    }

    // State
    property int sessionIndex: (typeof sessionModel !== "undefined" && sessionModel.lastIndex >= 0) ? sessionModel.lastIndex : 0
    property int userIndex: (typeof userModel !== "undefined" && userModel.lastIndex >= 0) ? userModel.lastIndex : 0
    property real ui: 0
    property bool revealSecret: false

    TextConstants { id: textConstants }

    FontLoader {
        id: veilaFont
        source: "fonts/GoogleSansFlex_72pt-Regular.ttf"
    }

    // Helpers
    ListView {
        id: sessionHelper
        model: typeof sessionModel !== "undefined" ? sessionModel : null
        currentIndex: root.sessionIndex
        visible: false
        delegate: Item { property string sName: model.name || "" }
    }

    ListView {
        id: userHelper
        model: typeof userModel !== "undefined" ? userModel : null
        currentIndex: root.userIndex
        visible: false
        delegate: Item {
            property string uName: model.realName || model.name || ""
            property string uLogin: model.name || ""
        }
    }

    // Background Image
    Image {
        id: bgImage
        anchors.fill: parent
        source: config.background || "wallpaper.png"
        fillMode: Image.PreserveAspectCrop
        asynchronous: false
        cache: false
    }

    // Subtle dark tint for contrast
    Rectangle {
        anchors.fill: parent
        color: "#000000"
        opacity: 0.15
    }

    // Fade-in animation on load
    Component.onCompleted: {
        fadeAnim.start()
        if (typeof keyboard !== "undefined") {
            keyboard.numLock = true
        }
    }

    Timer {
        interval: 200
        running: true
        onTriggered: passwordField.forceActiveFocus()
    }

    NumberAnimation {
        id: fadeAnim
        target: root
        property: "ui"
        from: 0
        to: 1
        duration: 800
        easing.type: Easing.OutCubic
    }

    // Clock & Date (Bottom-Right)
    Column {
        anchors.right: parent.right
        anchors.bottom: loginContainer.top
        anchors.rightMargin: 80 * s
        anchors.bottomMargin: 24 * s
        spacing: 4 * s
        opacity: root.ui

        Text {
            id: clockText
            anchors.right: parent.right
            text: Qt.formatTime(new Date(), "HH:mm")
            color: config.color || "#ecdfe5"
            font.family: veilaFont.name
            font.pixelSize: 84 * s
            font.weight: Font.Light

            Timer {
                interval: 1000
                running: true
                repeat: true
                onTriggered: clockText.text = Qt.formatTime(new Date(), "HH:mm")
            }
        }

        Text {
            anchors.right: parent.right
            text: Qt.formatDate(new Date(), "dddd, MMMM d")
            color: config.subtext || "#dcbed1"
            font.family: veilaFont.name
            font.pixelSize: 18 * s
            font.weight: Font.Normal
            opacity: 0.85
        }
    }

    // Login Container (Bottom-Right Underline Bar)
    Item {
        id: loginContainer
        anchors.right: parent.right
        anchors.bottom: bottomBar.top
        anchors.rightMargin: 80 * s
        anchors.bottomMargin: 50 * s
        width: 320 * s
        height: 44 * s
        opacity: root.ui

        // Text input field
        TextInput {
            id: passwordField
            anchors.left: parent.left
            anchors.right: arrowHint.left
            anchors.rightMargin: 12 * s
            anchors.verticalCenter: parent.verticalCenter
            color: config.color || "#ecdfe5"
            font.family: veilaFont.name
            font.pixelSize: 16 * s
            echoMode: root.revealSecret ? TextInput.Normal : TextInput.Password
            focus: true
            clip: true
            cursorVisible: true
            cursorDelegate: Rectangle {
                width: 1.5 * s
                color: config.accent || "#f4b2e2"
                visible: passwordField.focus
                SequentialAnimation on opacity {
                    loops: Animation.Infinite
                    NumberAnimation { from: 1; to: 0; duration: 500 }
                    NumberAnimation { from: 0; to: 1; duration: 500 }
                }
            }

            onTextEdited: errorMessage.text = ""
            Keys.onReturnPressed: doLogin()
            Keys.onEnterPressed: doLogin()

            // Dim Username Placeholder
            Text {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                text: (userHelper.currentItem && userHelper.currentItem.uLogin) ? userHelper.currentItem.uLogin : (typeof userModel !== "undefined" ? (userModel.lastUser || "pineapple") : "pineapple")
                color: config.subtext || "#dcbed1"
                opacity: (passwordField.text.length === 0) ? 0.45 : 0
                font.family: veilaFont.name
                font.pixelSize: 16 * s
                Behavior on opacity { NumberAnimation { duration: 150 } }
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.IBeamCursor
                onClicked: passwordField.forceActiveFocus()
            }
        }

        // Eye toggle icon
        Text {
            id: eyeToggle
            anchors.right: arrowHint.left
            anchors.rightMargin: 14 * s
            anchors.verticalCenter: parent.verticalCenter
            text: root.revealSecret ? "󰈈" : "󰈉"
            color: config.accent || "#f4b2e2"
            font.pixelSize: 16 * s
            opacity: passwordField.text.length > 0 ? 0.75 : 0
            visible: opacity > 0
            Behavior on opacity { NumberAnimation { duration: 150 } }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.revealSecret = !root.revealSecret
            }
        }

        // Interactive Click-to-Submit Arrow
        Text {
            id: arrowHint
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            text: "→"
            color: config.accent || "#f4b2e2"
            font.family: veilaFont.name
            font.pixelSize: 22 * s
            opacity: passwordField.text.length > 0 ? 1.0 : 0.35
            scale: arrowMa.containsMouse ? 1.15 : 1.0
            Behavior on scale { NumberAnimation { duration: 150; easing.type: Easing.OutBack } }
            Behavior on opacity { NumberAnimation { duration: 150 } }

            MouseArea {
                id: arrowMa
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: doLogin()
            }
        }

        // Underline Line
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 2 * s
            color: passwordField.activeFocus ? (config.focus_line || "#f4b2e2") : (config.idle_line || "#443840")
            Behavior on color { ColorAnimation { duration: 250 } }
        }

        // Error message text
        Text {
            id: errorMessage
            anchors.right: parent.right
            anchors.top: parent.bottom
            anchors.topMargin: 8 * s
            text: ""
            color: config.error || "#ffb4ab"
            font.family: veilaFont.name
            font.pixelSize: 12 * s
            font.weight: Font.Medium
        }
    }

    // Bottom Bar (Session Switcher & Power Controls)
    Item {
        id: bottomBar
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: 40 * s
        height: 36 * s
        opacity: root.ui * 0.85

        // Session Switcher (Bottom-Left)
        Item {
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            width: sessionRow.implicitWidth
            height: sessionRow.implicitHeight

            Row {
                id: sessionRow
                spacing: 8 * s
                opacity: sMa.containsMouse ? 1.0 : 0.65
                scale: sMa.containsMouse ? 1.05 : 1.0
                Behavior on opacity { NumberAnimation { duration: 150 } }
                Behavior on scale { NumberAnimation { duration: 150 } }

                Text {
                    text: "󰍹"
                    color: config.accent || "#f4b2e2"
                    font.pixelSize: 14 * s
                    anchors.verticalCenter: parent.verticalCenter
                }
                Text {
                    id: sessionLabel
                    text: (sessionHelper.currentItem && sessionHelper.currentItem.sName) ? sessionHelper.currentItem.sName : "Niri"
                    color: config.color || "#ecdfe5"
                    font.family: veilaFont.name
                    font.pixelSize: 13 * s
                    anchors.verticalCenter: parent.verticalCenter
                }
            }

            MouseArea {
                id: sMa
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    if (typeof sessionModel !== "undefined" && sessionModel.rowCount() > 0) {
                        root.sessionIndex = (root.sessionIndex + 1) % sessionModel.rowCount()
                    }
                }
            }
        }

        // Power Actions (Bottom-Right edge)
        Row {
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            spacing: 24 * s

            Text {
                text: "Suspend"
                color: config.subtext || "#dcbed1"
                opacity: susMa.containsMouse ? 1.0 : 0.5
                font.family: veilaFont.name
                font.pixelSize: 13 * s
                Behavior on opacity { NumberAnimation { duration: 150 } }
                MouseArea {
                    id: susMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: { if (typeof sddm !== "undefined") sddm.suspend() }
                }
            }

            Text {
                text: "Reboot"
                color: config.subtext || "#dcbed1"
                opacity: rebMa.containsMouse ? 1.0 : 0.5
                font.family: veilaFont.name
                font.pixelSize: 13 * s
                Behavior on opacity { NumberAnimation { duration: 150 } }
                MouseArea {
                    id: rebMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: { if (typeof sddm !== "undefined") sddm.reboot() }
                }
            }

            Text {
                text: "Shutdown"
                color: config.subtext || "#dcbed1"
                opacity: shutMa.containsMouse ? 1.0 : 0.5
                font.family: veilaFont.name
                font.pixelSize: 13 * s
                Behavior on opacity { NumberAnimation { duration: 150 } }
                MouseArea {
                    id: shutMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: { if (typeof sddm !== "undefined") sddm.powerOff() }
                }
            }
        }
    }

    // Login Action
    function doLogin() {
        var uname = (userHelper.currentItem && userHelper.currentItem.uLogin) ? userHelper.currentItem.uLogin : (typeof userModel !== "undefined" ? userModel.lastUser : "pineapple")
        if (typeof sddm !== "undefined") {
            sddm.login(uname, passwordField.text, root.sessionIndex)
        }
    }

    Connections {
        target: typeof sddm !== "undefined" ? sddm : null
        function onLoginFailed() {
            errorMessage.text = "Incorrect password"
            passwordField.text = ""
            passwordField.focus = true
            shakeAnim.start()
        }
    }

    SequentialAnimation {
        id: shakeAnim
        NumberAnimation { target: loginContainer; property: "anchors.rightMargin"; to: 90 * s; duration: 40 }
        NumberAnimation { target: loginContainer; property: "anchors.rightMargin"; to: 70 * s; duration: 40 }
        NumberAnimation { target: loginContainer; property: "anchors.rightMargin"; to: 86 * s; duration: 40 }
        NumberAnimation { target: loginContainer; property: "anchors.rightMargin"; to: 74 * s; duration: 40 }
        NumberAnimation { target: loginContainer; property: "anchors.rightMargin"; to: 80 * s; duration: 40 }
    }
}
