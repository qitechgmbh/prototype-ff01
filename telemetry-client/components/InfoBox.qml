import QtQuick
import QtQuick.Layouts

Item {
    id: root

    required property var backgroundColor;

    required property var descriptionText;
    required property var descriptionColor;

    required property var infoText;
    required property var infoColor;

    Rectangle {
        id: orderInfo

        anchors.fill: parent

        color: root.backgroundColor
        radius: 10

        ColumnLayout {
            anchors.verticalCenter: parent.verticalCenter

            anchors.left: parent.left
            anchors.leftMargin: 10

            Text {
                text: root.descriptionText
                color: root.descriptionColor
                font.pixelSize: 14
            }

            Text {
                text: root.infoText
                color: root.infoColor
                font.pixelSize: 20
            }
        }
    }
}