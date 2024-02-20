#pragma once

#include <QtCore/QLocale>
#include <QtCore/QVariant>
#include <QtWidgets/QApplication>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDialog>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpacerItem>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

QT_BEGIN_NAMESPACE

class Ui_EditVerDialog
{
public:
    QVBoxLayout *verticalLayout;
    QLabel *Title;
    QGridLayout *GLayout;
    QListWidget *MCList;
    QListWidget *OptfineList;
    QComboBox *ComboBox;
    QListWidget *ModList;
    QLabel *OptfineLabel;
    QLabel *MCLabel;
    QLabel *CfgLabel;
    QHBoxLayout *JavaHLayout;
    QLabel *JavaLabel;
    QLineEdit *JavaEdit;
    QToolButton *JavaBtn;
    QHBoxLayout *HeightHLayout;
    QLabel *HeightLabel;
    QSpinBox *HeightEdit;
    QHBoxLayout *WidthHLayout;
    QLabel *WidthLabel;
    QSpinBox *WidthEdit;
    QHBoxLayout *XmsHLayout;
    QLabel *XmsLabel;
    QLineEdit *XmsEdit;
    QHBoxLayout *XmxHLayout;
    QLabel *XmxLabel;
    QLineEdit *XmxEdit;
    QHBoxLayout *ArgsHLayout;
    QLabel *ArgsLabel;
    QLineEdit *ArgsEdit;
    QToolButton *ArgsButton;
    QHBoxLayout *MainHLayout;
    QPushButton *DelBtn;
    QSpacerItem *HSpacer;
    QPushButton *CancelBtn;
    QPushButton *DoneBtn;

    void setupUi(QDialog *EditVerDialog)
    {
        if (EditVerDialog->objectName().isEmpty())
            EditVerDialog->setObjectName("EditVerDialog");
        EditVerDialog->resize(900, 600);
        EditVerDialog->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        verticalLayout = new QVBoxLayout(EditVerDialog);
        verticalLayout->setObjectName("verticalLayout");
        Title = new QLabel(EditVerDialog);
        Title->setObjectName("Title");
        Title->setTextFormat(Qt::MarkdownText);

        verticalLayout->addWidget(Title);

        GLayout = new QGridLayout();
        GLayout->setObjectName("GLayout");
        MCList = new QListWidget(EditVerDialog);
        MCList->setObjectName("MCList");

        GLayout->addWidget(MCList, 1, 2, 1, 1);

        OptfineList = new QListWidget(EditVerDialog);
        OptfineList->setObjectName("OptfineList");

        GLayout->addWidget(OptfineList, 1, 4, 1, 1);

        ComboBox = new QComboBox(EditVerDialog);
        ComboBox->setObjectName("ComboBox");

        GLayout->addWidget(ComboBox, 0, 3, 1, 1);

        ModList = new QListWidget(EditVerDialog);
        ModList->setObjectName("ModList");

        GLayout->addWidget(ModList, 1, 3, 1, 1);

        OptfineLabel = new QLabel(EditVerDialog);
        OptfineLabel->setObjectName("OptfineLabel");
        OptfineLabel->setTextFormat(Qt::MarkdownText);

        GLayout->addWidget(OptfineLabel, 0, 4, 1, 1);

        MCLabel = new QLabel(EditVerDialog);
        MCLabel->setObjectName("MCLabel");
        MCLabel->setTextFormat(Qt::MarkdownText);

        GLayout->addWidget(MCLabel, 0, 2, 1, 1);


        verticalLayout->addLayout(GLayout);

        CfgLabel = new QLabel(EditVerDialog);
        CfgLabel->setObjectName("CfgLabel");
        CfgLabel->setTextFormat(Qt::MarkdownText);

        verticalLayout->addWidget(CfgLabel);

        JavaHLayout = new QHBoxLayout();
        JavaHLayout->setObjectName("JavaHLayout");
        JavaLabel = new QLabel(EditVerDialog);
        JavaLabel->setObjectName("JavaLabel");
        QSizePolicy sizePolicy(QSizePolicy::Policy::Preferred, QSizePolicy::Policy::Preferred);
        sizePolicy.setHorizontalStretch(0);
        sizePolicy.setVerticalStretch(0);
        sizePolicy.setHeightForWidth(JavaLabel->sizePolicy().hasHeightForWidth());
        JavaLabel->setSizePolicy(sizePolicy);
        JavaLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        JavaHLayout->addWidget(JavaLabel);

        JavaEdit = new QLineEdit(EditVerDialog);
        JavaEdit->setObjectName("JavaEdit");
        sizePolicy.setHeightForWidth(JavaEdit->sizePolicy().hasHeightForWidth());
        JavaEdit->setSizePolicy(sizePolicy);
        JavaEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        JavaHLayout->addWidget(JavaEdit);

        JavaBtn = new QToolButton(EditVerDialog);
        JavaBtn->setObjectName("JavaBtn");
        JavaBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        JavaHLayout->addWidget(JavaBtn);


        verticalLayout->addLayout(JavaHLayout);

        HeightHLayout = new QHBoxLayout();
        HeightHLayout->setObjectName("HeightHLayout");
        HeightLabel = new QLabel(EditVerDialog);
        HeightLabel->setObjectName("HeightLabel");
        HeightLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        HeightHLayout->addWidget(HeightLabel);

        HeightEdit = new QSpinBox(EditVerDialog);
        HeightEdit->setObjectName("HeightEdit");
        sizePolicy.setHeightForWidth(HeightEdit->sizePolicy().hasHeightForWidth());
        HeightEdit->setSizePolicy(sizePolicy);
        HeightEdit->setMinimum(100);
        HeightEdit->setMaximum(2147483647);

        HeightHLayout->addWidget(HeightEdit);


        verticalLayout->addLayout(HeightHLayout);

        WidthHLayout = new QHBoxLayout();
        WidthHLayout->setObjectName("WidthHLayout");
        WidthLabel = new QLabel(EditVerDialog);
        WidthLabel->setObjectName("WidthLabel");
        WidthLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        WidthHLayout->addWidget(WidthLabel);

        WidthEdit = new QSpinBox(EditVerDialog);
        WidthEdit->setObjectName("WidthEdit");
        sizePolicy.setHeightForWidth(WidthEdit->sizePolicy().hasHeightForWidth());
        WidthEdit->setSizePolicy(sizePolicy);
        WidthEdit->setMinimum(100);
        WidthEdit->setMaximum(2147483647);

        WidthHLayout->addWidget(WidthEdit);


        verticalLayout->addLayout(WidthHLayout);

        XmsHLayout = new QHBoxLayout();
        XmsHLayout->setObjectName("XmsHLayout");
        XmsLabel = new QLabel(EditVerDialog);
        XmsLabel->setObjectName("XmsLabel");
        XmsLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmsHLayout->addWidget(XmsLabel);

        XmsEdit = new QLineEdit(EditVerDialog);
        XmsEdit->setObjectName("XmsEdit");
        sizePolicy.setHeightForWidth(XmsEdit->sizePolicy().hasHeightForWidth());
        XmsEdit->setSizePolicy(sizePolicy);
        XmsEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmsHLayout->addWidget(XmsEdit);


        verticalLayout->addLayout(XmsHLayout);

        XmxHLayout = new QHBoxLayout();
        XmxHLayout->setObjectName("XmxHLayout");
        XmxLabel = new QLabel(EditVerDialog);
        XmxLabel->setObjectName("XmxLabel");
        XmxLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmxHLayout->addWidget(XmxLabel);

        XmxEdit = new QLineEdit(EditVerDialog);
        XmxEdit->setObjectName("XmxEdit");
        sizePolicy.setHeightForWidth(XmxEdit->sizePolicy().hasHeightForWidth());
        XmxEdit->setSizePolicy(sizePolicy);
        XmxEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmxHLayout->addWidget(XmxEdit);


        verticalLayout->addLayout(XmxHLayout);

        ArgsHLayout = new QHBoxLayout();
        ArgsHLayout->setObjectName("ArgsHLayout");
        ArgsLabel = new QLabel(EditVerDialog);
        ArgsLabel->setObjectName("ArgsLabel");

        ArgsHLayout->addWidget(ArgsLabel);

        ArgsEdit = new QLineEdit(EditVerDialog);
        ArgsEdit->setObjectName("ArgsEdit");

        ArgsHLayout->addWidget(ArgsEdit);

        ArgsButton = new QToolButton(EditVerDialog);
        ArgsButton->setObjectName("ArgsButton");

        ArgsHLayout->addWidget(ArgsButton);


        verticalLayout->addLayout(ArgsHLayout);

        MainHLayout = new QHBoxLayout();
        MainHLayout->setObjectName("MainHLayout");
        DelBtn = new QPushButton(EditVerDialog);
        DelBtn->setObjectName("DelBtn");
        DelBtn->setMinimumSize(QSize(100, 50));

        MainHLayout->addWidget(DelBtn);

        HSpacer = new QSpacerItem(40, 20, QSizePolicy::Policy::Expanding, QSizePolicy::Policy::Minimum);

        MainHLayout->addItem(HSpacer);

        CancelBtn = new QPushButton(EditVerDialog);
        CancelBtn->setObjectName("CancelBtn");
        CancelBtn->setMinimumSize(QSize(100, 50));

        MainHLayout->addWidget(CancelBtn);

        DoneBtn = new QPushButton(EditVerDialog);
        DoneBtn->setObjectName("DoneBtn");
        DoneBtn->setMinimumSize(QSize(100, 50));

        MainHLayout->addWidget(DoneBtn);


        verticalLayout->addLayout(MainHLayout);


        retranslateUi(EditVerDialog);

        QMetaObject::connectSlotsByName(EditVerDialog);
    } // setupUi

    void retranslateUi(QDialog *EditVerDialog)
    {
        EditVerDialog->setWindowTitle(QCoreApplication::translate("EditVerDialog", "Edit a Minecraft Version", nullptr));
        Title->setText(QCoreApplication::translate("EditVerDialog", "# Edit a Minecraft Version", nullptr));
        ComboBox->setPlaceholderText(QCoreApplication::translate("EditVerDialog", "Mod Loader", nullptr));
        OptfineLabel->setText(QCoreApplication::translate("EditVerDialog", "### Optfine", nullptr));
        MCLabel->setText(QCoreApplication::translate("EditVerDialog", "### Minecraft Version", nullptr));
        CfgLabel->setText(QCoreApplication::translate("EditVerDialog", "### Configs", nullptr));
        JavaLabel->setText(QCoreApplication::translate("EditVerDialog", "Java Path", nullptr));
        JavaBtn->setText(QCoreApplication::translate("EditVerDialog", "...", nullptr));
        HeightLabel->setText(QCoreApplication::translate("EditVerDialog", "Height", nullptr));
        WidthLabel->setText(QCoreApplication::translate("EditVerDialog", "Width", nullptr));
        XmsLabel->setText(QCoreApplication::translate("EditVerDialog", "Minimum Memory", nullptr));
        XmxLabel->setText(QCoreApplication::translate("EditVerDialog", "Maximum Memory", nullptr));
        ArgsLabel->setText(QCoreApplication::translate("EditVerDialog", "Launch Arguments ( Dangerous )", nullptr));
        ArgsButton->setText(QCoreApplication::translate("EditVerDialog", "...", nullptr));
        DelBtn->setText(QCoreApplication::translate("EditVerDialog", "Delete", nullptr));
        CancelBtn->setText(QCoreApplication::translate("EditVerDialog", "Cancel", nullptr));
        DoneBtn->setText(QCoreApplication::translate("EditVerDialog", "Done", nullptr));
    } // retranslateUi

};

namespace Ui {
    class EditVerDialog: public Ui_EditVerDialog {};
} // namespace Ui

QT_END_NAMESPACE
