import curses
import time
import math

#  Example Layout:
#   _________________________
#  | 0. Naman 3:37 / 5:32    |
#  > 1. Evan 1:32 / 1:32     <   
#  |    L Grace 3:13 / 5:31  |
#  |    L Naman 1:24 / 5:32  |
#  |    L Cici 4:02 / 5:50   |
#  | 2. Cici 1:32 / 5:50     |
#  | *                       |
#  | *                       |
#  | *                       |
#  | 10. Evan 0:00 / 1:32    |
#  | ------------------------|
#  |     Total Time: 13:37   |
#  L -------------------------
#  Add Speaker: Nam_
#               Naman

###############################
# Let's define some things :) #
###############################

do_hints = True
prioritize_shy_speakers = True


def format_time(t):
    return (str(t/60) + ':' + ('0' if t%60 < 10 else '' ) + str(t%60))

def detect_partial_match(a,b):
    if len(a) == 0 or len(b) < len(a):
        return False
    else:
        for i in range(len(a)):
            if a[i] != b[i]:
                return False
        return True

class Speaker:
    
        # Static Varaiables
        roster = []

        # Static Methods
        @classmethod
        def search_roster(cls, name, allow_creation=False, matching_function=(lambda a,b: a == b)):
            for speaker in cls.roster:
                if (matching_function(name.upper(),speaker.get_name())):
                    return speaker
            if (allow_creation):
                return cls(name)
            else:
                return False

        # Constructors
        def __init__(self, name):
            self.__name = name.upper()
            self.__total_speaking_time = 0
            self.__number_of_speaches = 0
            self.__number_of_resonses = 0
            Speaker.roster += [self]

        # Getters
        def get_name(self):
            return self.__name
        
        def get_speaking_time(self):
            return self.__total_speaking_time

        # Setters
        def _tick_speaking_time(self): # This is called automatically by Speach.tick_speach_time(self), and should really not be called by anything else
            self.__total_speaking_time += 1

class Speach:

    # Static Variables
    speaking_order = []
    current_speach_index = 0
    current_response_index = -1
    net_duration = 0

    # Static Methods
    @classmethod
    def get_current_speach_index(cls, include_responses=False):
        ret = cls.current_speach_index
        if include_responses:
            for i in range(cls.current_speach_index):
                ret += len(Speach.speaking_order[i].get_responses())
        return ret
    
    @classmethod
    def get_current_response_index(cls):
        return cls.current_response_index

    @classmethod
    def get_number_of_ones(cls):
        return len(cls.speaking_order)

    @classmethod
    def get_total_number_of_speaches(cls):
        number_of_responses = 0
        for speach in cls.speaking_order:
            number_of_responses += len(speach.get_responses())
        return cls.get_number_of_ones() + number_of_responses

    @classmethod
    def get_current_speach(cls):
        return cls.speaking_order[cls.current_speach_index]

    @classmethod
    def goto_next_speach(cls):
        if cls.current_response_index+1 < len(cls.get_current_speach().get_responses()):
            cls.current_response_index += 1
        elif cls.get_current_speach_index() < cls.get_number_of_ones()-1:
            cls.current_speach_index += 1
            cls.current_response_index = -1
    
    @classmethod
    def goto_previous_speach(cls):
        if cls.current_response_index > -1:
            cls.current_response_index -= 1
        elif cls.current_speach_index > 0:
            cls.current_speach_index -= 1
            cls.current_response_index = len(cls.get_current_speach().get_responses())-1
    
    @classmethod
    def tick_clock(cls):
        if cls.get_current_response_index() < 0:
            cls.get_current_speach().tick_speach_time()
        else:
            cls.get_current_speach().get_responses()[cls.get_current_response_index()].tick_speach_time() 

    # Constructors
    def __init__(self, name, index='auto'):
        self.__speaker = Speaker.search_roster(name, allow_creation=True)
        self.__duration = 0
        self.__responses = []
        if index == 'auto':
            self.__index = len(Speach.speaking_order)
            Speach.speaking_order += [self]
        else:
            self.__index = index
        
    # Getters
    def get_speaker(self):
        return self.__speaker

    def get_duration(self):
        return self.__duration

    def get_number(self):
        return self.__index

    def get_responses(self):
        return self.__responses
    
    # Setters
    def tick_speach_time(self):
        self.__duration += 1
        self.get_speaker()._tick_speaking_time()
        Speach.net_duration += 1

    def respond(self, name):
        self.__responses += [Speach(name,index=len(self.get_responses()))]

#Speach('Cici')
#Speach('Naman')
#Speach('Evan')
#Speach('Cici')
#Speach('Naman')
#Speach('Evan')
#Speach('Grace')

#Speach('Naman')
#Speach('Evan')
#Speach.speaking_order[2].respond('Marcus')
#Speach.speaking_order[2].respond('Nick')
#Speach.speaking_order[4].respond('Marcus')
#Speach.speaking_order[4].respond('Nick')
#Speach.speaking_order[4].respond('Lily')
#Speach.speaking_order[4].respond('Daniel')

def main(stdscr):

    # Setup
    #try:
    #    curses.curs_set(False)
    #except curses.error:
    #    pass
    
    while(True):
        
        #try:
        APP_WIDTH = 50
        APP_X_POS = (curses.COLS-APP_WIDTH)/2
        APP_Y_POS = 10

        SPEAKERS_HEIGHT = 30
        SPEAKERS_Y_POS = 1

        LAST_SPEAKER_HEIGHT = 4
        LAST_SPEAKER_Y_POS = SPEAKERS_Y_POS+SPEAKERS_HEIGHT-LAST_SPEAKER_HEIGHT
        
        CLOCK_HEIGHT = 1
        CLOCK_Y_POS = SPEAKERS_Y_POS + SPEAKERS_HEIGHT + 1
        
        PROMPT_HEIGHT = 1
        PROMPT_Y_POS = CLOCK_Y_POS + CLOCK_HEIGHT + 1

        SUBPROMPT_HEIGHT = 1
        SUBPROMPT_Y_POS = PROMPT_Y_POS + PROMPT_HEIGHT

        stdscr.clear()
        stdscr.addch(APP_Y_POS-1,APP_X_POS-1, curses.ACS_ULCORNER)
        stdscr.addch(APP_Y_POS-1,APP_X_POS+APP_WIDTH, curses.ACS_URCORNER)
        stdscr.addch(APP_Y_POS+SPEAKERS_Y_POS+SPEAKERS_HEIGHT,APP_X_POS-1, curses.ACS_LTEE)
        stdscr.addch(APP_Y_POS+SPEAKERS_Y_POS+SPEAKERS_HEIGHT,APP_X_POS+APP_WIDTH, curses.ACS_RTEE)
        stdscr.addch(APP_Y_POS+CLOCK_Y_POS, APP_X_POS-1, curses.ACS_VLINE)
        stdscr.addch(APP_Y_POS+CLOCK_Y_POS, APP_X_POS+APP_WIDTH, curses.ACS_VLINE)
        stdscr.addch(APP_Y_POS+CLOCK_Y_POS+CLOCK_HEIGHT, APP_X_POS-1, curses.ACS_LLCORNER)
        stdscr.addch(APP_Y_POS+CLOCK_Y_POS+CLOCK_HEIGHT, APP_X_POS+APP_WIDTH, curses.ACS_LRCORNER)
        for x in range(APP_X_POS,APP_X_POS+APP_WIDTH):
            stdscr.addch(APP_Y_POS-1,x, curses.ACS_HLINE)
            stdscr.addch(APP_Y_POS+SPEAKERS_Y_POS+SPEAKERS_HEIGHT,x, curses.ACS_HLINE)
            stdscr.addch(APP_Y_POS+CLOCK_Y_POS+CLOCK_HEIGHT, x, curses.ACS_HLINE)
        for y in range(APP_Y_POS, APP_Y_POS+1+SPEAKERS_HEIGHT):
            stdscr.addch(y, APP_X_POS-1, curses.ACS_VLINE)
            stdscr.addch(y, APP_X_POS+APP_WIDTH, curses.ACS_VLINE)

        if do_hints:
            stdscr.addstr(APP_Y_POS+0,APP_X_POS+APP_WIDTH+2,'C-n Go to next speaker')
            stdscr.addstr(APP_Y_POS+1,APP_X_POS+APP_WIDTH+2,'C-b Go to previous speaker')
            stdscr.addstr(APP_Y_POS+2,APP_X_POS+APP_WIDTH+2,'C-p Pause the clock')
            stdscr.addstr(APP_Y_POS+2,APP_X_POS+APP_WIDTH+2,'C-h Toggle Hints')
            stdscr.addstr(APP_Y_POS+3,APP_X_POS+APP_WIDTH+2,'C-r Re-render')
            stdscr.addstr(APP_Y_POS+4,APP_X_POS+APP_WIDTH+2,'C-d Exit')

        #stdscr.addstr(0,0, str(Speach.get_total_number_of_speaches()))
        
        stdscr.refresh()
        
        speakers_box = curses.newpad(1000, APP_WIDTH)
        last_speaker_box = curses.newwin(LAST_SPEAKER_HEIGHT, APP_WIDTH, APP_Y_POS+LAST_SPEAKER_Y_POS, APP_X_POS)
        def update_speakers_box():

            def add_speaker(header, speach, highlight=False,box=speakers_box):
                speaker_name = speach.get_speaker().get_name().capitalize()
                speaker_time = format_time(speach.get_duration()) + ' / ' + format_time(speach.get_speaker().get_speaking_time())
                box.addch(curses.ACS_RARROW if highlight else ' ')
                for ch in header:
                    box.addch(ch)
                box.addstr(speaker_name, curses.A_BOLD if highlight else 0)
                for x in range(len(header) + len(speaker_name) + 2, APP_WIDTH-len(speaker_time)-2):
                    box.addch('.')
                
                box.addstr(speaker_time)
                box.addch(' ')
                box.addch(curses.ACS_LARROW if highlight else ' ')

            make_seperate_box_for_last_speach = Speach.get_total_number_of_speaches()-Speach.get_current_speach_index()-Speach.get_current_response_index() > SPEAKERS_HEIGHT+2 # I'm not entirely sure where this 2 comes from, and it worries me that I can't account for it, but it seems to make it work right...

            speakers_box.clear()
            for speach in Speach.speaking_order:
                highlight_this_or_a_response = speach.get_number() == Speach.get_current_speach_index()
                add_speaker(' ' + str(speach.get_number()) + '. ', speach, highlight_this_or_a_response and Speach.current_response_index == -1)
                speakers_box.addch('\n')
                if len(speach.get_responses()) != 0:
                    for response in speach.get_responses()[:-1]:
                        add_speaker([' ',' ',' ',' ',curses.ACS_LTEE], response, highlight_this_or_a_response and Speach.current_response_index == response.get_number())
                        speakers_box.addch('\n')
                    response = speach.get_responses()[-1] 
                    add_speaker([' ',' ',' ',' ',curses.ACS_LLCORNER], response, highlight_this_or_a_response and Speach.current_response_index == response.get_number())
                    speakers_box.addch('\n')
            speakers_box.refresh(Speach.get_current_speach_index(include_responses=True)+Speach.get_current_response_index()+1 ,0, APP_Y_POS, APP_X_POS, APP_Y_POS+SPEAKERS_HEIGHT-(LAST_SPEAKER_HEIGHT if make_seperate_box_for_last_speach else 0), APP_X_POS+APP_WIDTH)
            if make_seperate_box_for_last_speach:
                last_speaker_box.clear()
                last_speaker_box.addstr('  *\n  *\n  *\n')
                add_speaker(' ' + str(len(Speach.speaking_order)-1) + '. ',Speach.speaking_order[-1],highlight=False,box=last_speaker_box)
                last_speaker_box.refresh()

        update_speakers_box()
        
        CLOCK_LABEL_TEXT = 'Total Time Elapsed: '
        CLOCK_LABEL_WIDTH = len(CLOCK_LABEL_TEXT)
        clock_label = curses.newwin(CLOCK_HEIGHT, CLOCK_LABEL_WIDTH+1, APP_Y_POS+CLOCK_Y_POS, APP_X_POS)
        clock_label.addstr(CLOCK_LABEL_TEXT, curses.A_BOLD)
        clock_label.refresh()

        clock_face = curses.newwin(CLOCK_HEIGHT, APP_WIDTH-CLOCK_LABEL_WIDTH, APP_Y_POS+CLOCK_Y_POS, APP_X_POS+CLOCK_LABEL_WIDTH)
        def update_clock():
            clock_face.clear()
            clock_face.addstr(format_time(Speach.net_duration))
            clock_face.refresh()
        update_clock()

        PROMPT_TEXT = 'Add Speaker: '
        PROMPT_WIDTH = len(PROMPT_TEXT)
        prompt_win = curses.newwin(PROMPT_HEIGHT, PROMPT_WIDTH+1, APP_Y_POS+PROMPT_Y_POS, APP_X_POS)
        prompt_win.addstr(PROMPT_TEXT)
        prompt_win.refresh()

        #INPUT_CURSOR = '_'
        input_win = curses.newwin(PROMPT_HEIGHT, APP_WIDTH-PROMPT_WIDTH, APP_Y_POS+PROMPT_Y_POS, APP_X_POS+PROMPT_WIDTH)
        #input_win.addch(INPUT_CURSOR)

        subprompt_win = curses.newwin(SUBPROMPT_HEIGHT, APP_WIDTH, APP_Y_POS+SUBPROMPT_Y_POS, APP_X_POS)

        input_content = ''
        autocomplete_guess = ''
        def subprompt_autocomplete():
            speaker_guess = Speaker.search_roster(input_content, allow_creation=False, matching_function=detect_partial_match)
            subprompt_win.clear()
            if speaker_guess == False:
                autocomplete_guess = ''
            else:
                autocomplete_guess = speaker_guess.get_name().capitalize()
                subprompt_win.addstr(0, PROMPT_WIDTH, autocomplete_guess)
            subprompt_win.refresh()
            return autocomplete_guess

        mode = 0
        input_win.nodelay(True)
        most_recent_recorded_time = math.trunc(time.time())

        while(True):
            
            if Speach.speaking_order != []:
                current_time =  math.trunc(time.time())
                
                if current_time > most_recent_recorded_time:
                    Speach.tick_clock()
                    update_speakers_box()
                    update_clock()
                    most_recent_recorded_time = current_time
            try:
                key = input_win.getkey()
                ord(key) # Just making sure that this function works, because for some reason it occasionally gets strings of length 10
            except:
                continue
            
            if mode == 0:
                if ord(key) == 127:
                    input_content = input_content[:-1]
                    autocomplete_guess = subprompt_autocomplete()
                elif key == ' ' or (ord(key) >= ord('a') and ord(key) <= ord('z')) or (ord(key) >= ord('A') and ord(key) <= ord('Z')) :
                    input_content += key
                    autocomplete_guess=subprompt_autocomplete()
                elif ord(key) == 4: # Ctrl-D (Terminate program)
                    return
                elif ord(key) == 9: # Tab
                    input_content = autocomplete_guess
                elif ord(key) == 10 and input_content != '': # Return
                    autocomplete_guess = ''
                    subprompt_win.clear()
                    subprompt_win.addstr('Type (1 or 2)?')
                    subprompt_win.refresh()
                    mode = 1
                elif ord(key) == 16: # Ctrl-P (Pause)
                    subprompt_win.clear()
                    subprompt_win.addstr('Clock Paused: Press any key to continue...', curses.A_BOLD)
                    subprompt_win.refresh()
                    while(True):
                        try:
                            input_win.getkey()
                        except:
                            continue
                        subprompt_autocomplete()
                        break
                elif ord(key) == 18: # Ctrl-R (Re-render)
                    break
                elif ord(key) == 8: # Ctrl-H (Toggle Hints)
                    global do_hints
                    do_hints = not do_hints
                    break
                elif ord(key) == 14: # Ctrl-N (Next)
                    Speach.goto_next_speach()
                    update_speakers_box()
                elif ord(key) == 2: # Ctrl-B (Go back to previous speach)
                    Speach.goto_previous_speach()
                    update_speakers_box()
                else:
                    subprompt_win.clear()
                    subprompt_win.addstr(str(ord(key)))
                    subprompt_win.refresh()
                
                subprompt_win.clear()
                
                input_win.clear()
                input_win.addstr(input_content)
                #input_win.addch(INPUT_CURSOR)
                input_win.refresh()

            elif mode == 1:
                if key == '1':
                    Speach(input_content)
                    update_speakers_box()

                    subprompt_win.clear()
                    subprompt_win.refresh()
                    input_content = ''
                    input_win.clear()
                    input_win.refresh()
                    mode = 0

                elif key == '2':
                    Speach.get_current_speach().respond(input_content)
                    update_speakers_box()

                    subprompt_win.clear()
                    subprompt_win.refresh()
                    input_content = ''
                    input_win.clear()
                    input_win.refresh()
                    mode = 0
                elif ord(key) == 27: # Escape
                    subprompt_win.clear()
                    subprompt_win.refresh()
                    input_content = ''
                    input_win.clear()
                    input_win.refresh()
                    mode = 0


curses.wrapper(main)