# coding: utf-8

from cdg.get import *

# Setup



# Funcs

def get_data(file, csv=True, name='output'):
    """
    Save DataFrame to csv or xlsx file.

    :param file: DataFrame
    :param csv: Bool
    :param name: str
    :return:
    """
    get_time()
    if type(file) == pd.DataFrame:
        df = file
    else:
        print('Error: file is not a DataFrame!')
        return
    if csv:
        df.to_csv(f'{files_path}/{name}_{date}_{time}.csv')
    elif not csv:
        df.to_excel(f'{files_path}/{name}_{date}_{time}.xlsx')
    else:
        print('Error: weird \'exc\' argument value!')
