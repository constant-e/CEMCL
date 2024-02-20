#pragma once

#include <iostream>
#include <QDialog>
#include <QObject>

QT_BEGIN_NAMESPACE
namespace Ui { class EditAccDialog; }
QT_END_NAMESPACE

class EditAccDialog : public QDialog
{
    Q_OBJECT

public slots:

public:
    EditAccDialog(QWidget *parent = nullptr);
    ~EditAccDialog();

private:
    Ui::EditAccDialog *UI;
    
};
