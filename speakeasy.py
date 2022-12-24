import curses
import time
import math

#  Example Layout:
#   _________________________
#  | 0. Naman 3:37 / 5:32    |
#  |>1. Evan 1:32 / 1:32    <|   
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

def format_time(t): 
    # t :: Int such that t >= 0
    return (str(t/60) + ':' + ('0' if t%60 < 10 else '' ) + str(t%60))

def detect_partial_match(a,b): 
    # String, String -> Bool
    if len(a) == 0 or len(b) < len(a):
        return False
    else:
        for i in range(len(a)):
            if a[i] != b[i]:
                return False
        return True

def sorted_insert(list, new_element, comparison_function):
    # list :: [t]
    # new_element :: [t]
    # comparison_function :: t, t -> Bool
    i = 0
    len_list = len(list)
    while(True):
        if i >= len_list:
            list.append(new_element)
            return
        elif comparison_function(new_element, list[i]):
            list.insert(i, new_element)
            return
        i += 1
    
    
def _audit_global_functions():
    
    lst = [1,2,3,5]
    sorted_insert(lst, 4, comparison_function=lambda x,y: x < y)
    assert(lst==[1,2,3,4,5])

    lst = [1,2,3,5]
    sorted_insert(lst, 200, comparison_function=lambda x,y: x < y)
    assert(lst==[1,2,3,5,200])

    lst = [1,2,3,5]
    sorted_insert(lst, -200, comparison_function=lambda x,y: x < y)
    assert(lst==[-200, 1,2,3,5])

    lst = [5,4,2,1]
    sorted_insert(lst, 3, comparison_function=lambda x,y: x > y)
    assert(lst==[5,4,3,2,1])

_audit_global_functions()

class Speaker: 
    # Constructors
    def __init__(self, name):
        self.__name = name.upper()
        self.__total_speaking_time = 0
        self.__number_of_speaches_given = 0

    # Getters
    def get_name(self):
        return self.__name

    def name_is(self, query, matching_function=(lambda query, actual_uppercase_name: query.uppercase() == actual_uppercase_name)):
        return matching_function(query, self.get_name())
    
    def get_total_speaking_time(self):
        return self.__total_speaking_time

    def get_number_of_speaches_given(self):
        return self.__number_of_speaches_given

    # Pseudoprivate Methods
    def _tick_speaking_time(self):
        self.__total_speaking_time += 1

    def _increment_number_of_speaches(self):
        self.__number_of_speaches_given += 1

class Speach:

    # Constructors
    def __init__(self, speaker, is_response): # speaker: Speaker
        speaker._increment_number_of_speaches()
        self.__speaker = speaker
        self.__duration = 0
        self.IS_RESPONSE = is_response

    # Getters
    def get_speaker(self):
        return self.__speaker

    def get_duration(self):
        return self.__duration

    def get_responses(self):
        return self.__responses
    
    # Pseudoprivate Methods
    def _tick_speach_time(self):
        self.__duration += 1
        self.get_speaker()._tick_speaking_time()

class Discussion:

    # Constructors
    def __init__(self, mover):
        self.__speakers = [mover]
        self.__current_speach = Speach(speaker=mover, is_response=False)
        self.__upcoming_speaches = []
        self.__number_of_upcoming_new_points = 0
        self.__number_of_upcoming_responses = 0
        self.__past_speaches = []
        self.__duration = 0
        self.__priority_mode = 2
        # 0 - No priority
        # 1 - Speakers who have raised the least number of points get to go first (not implemented yet)
        # 2 - Those who have spoken for the lowest amount of time get to go first

    # Getters
    def get_current_speach(self):
        return self.__current_speach

    def take_upcoming_speaches(self, how_many=False, remove_whats_been_taken=False): # how_many: False or Int>0
        # Returns how_many of the upcoming speaches in order
        # If how_many=False or if how_many is greater than the number of upcoming speaches, then it returns all of the upcoming speaches
        # If remove_whats_been_taken=True, then all the speaches returned by this function will be removed from the list of upcoming speaches. Use this option carefully and sparingly!
        
        if self.get_priority_mode() == 2:
            def add_element(elem, lst):
                sorted_insert(lst, elem, (lambda s1,s2: s1.get_speaker().get_total_speaking_time() < s2.get_speaker().get_total_speaking_time()))
        else:
            def add_element(elem, lst):
                lst.append(elem)

        if how_many == False:
            number_of_responses_remaining = self.get_number_of_upcoming_responses()
            number_of_new_points_remaining = self.get_number_of_upcoming_new_points()
        else:
            number_of_responses_remaining = min(self.get_number_of_upcoming_responses(), how_many)
            number_of_new_points_remaining = min(self.get_number_of_upcoming_new_points(), how_many-number_of_responses_remaining)
        responses = []
        new_points = []
        if remove_whats_been_taken:
            untaken = []
        for speach in self.__upcoming_speaches:
            if speach.IS_RESPONSE:
                if number_of_responses_remaining > 0:
                    add_element(speach,responses)
                    number_of_responses_remaining -= 1
                elif remove_whats_been_taken:
                    untaken.append(speach)
            else:
                if number_of_new_points_remaining > 0:
                    add_element(speach,new_points)
                    number_of_new_points_remaining -= 1
                elif remove_whats_been_taken:
                    untaken.append(speach)
        if remove_whats_been_taken:
            self.__upcoming_speaches = untaken
        return responses+new_points
    
    def get_number_of_upcoming_new_points(self):
        return self.__number_of_upcoming_new_points

    def get_number_of_upcoming_responses(self):
        return self.__number_of_upcoming_responses

    def get_number_of_upcoming_speaches(self):
        return self.get_number_of_upcoming_new_points()+self.get_number_of_upcoming_responses()

    def get_past_speaches(self):
        return self.__past_speaches

    def get_duration(self):
        return self.__duration

    def get_priority_mode(self):
        return self.__priority_mode

    def find_speaker(self, name, matching_function=(lambda a,b: a==b)): # name: String, matching_function: Function (String String -> Bool)
        # Searches through the speakers currently listed to find one matching name where a "match" is defined by matching_function
        # Matching function should take two strings and return True if the strings match and False otherwise
        # It should be noted that matching_function is not necessarily commutative, and it is specifically asking if the first argument matches the second one (rather than asking if the second argument matches the first one). This is a subtle distinction but just keep in mind the way the matching_function is used. The first argument passed is our search and the second argument is the alledged actual name we are looking for. In other words, the first argument may be incomplete, or it may have typoes, or whatever other things I or someone else wants to impliment. The second argument is the flawless platonic ideal
        for speaker in self.__speakers:
            if (matching_function(name.upper(), speaker.get_name())):
                return speaker
        return False
    
    # Setters
    def add_speach(self, speaker_name, is_response):
        speaker = self.find_speaker(speaker_name)
        if speaker == False:
            speaker = Speaker(speaker_name)
            self.__speakers.append(speaker)
        self.__upcoming_speaches.append(Speach(speaker, is_response))
        if is_response:
            self.__number_of_upcoming_responses += 1
        else:
            self.__number_of_upcoming_new_points += 1

    def goto_next_speach(self):
        upcoming_speaches = self.take_upcoming_speaches(how_many=1, remove_whats_been_taken=True)
        if upcoming_speaches != []:
            self.__past_speaches.append(self.__current_speach)
            self.__current_speach = upcoming_speaches[0]
            if self.__current_speach.IS_RESPONSE:
                self.__number_of_upcoming_responses -= 1
            else:
                self.__number_of_upcoming_new_points -= 1

    def goto_previous_speach(self):
        if self.__past_speaches != []:
            if self.__current_speach.IS_RESPONSE:
                self.__number_of_upcoming_responses += 1
            else:
                self.__number_of_upcoming_new_points += 1
            self.__upcoming_speaches.insert(0, self.get_current_speach())
            self.__current_speach = self.__past_speaches[-1]
            self.__past_speaches = self.__past_speaches[:-1]

    def tick_clock(self):
        self.__duration += 1
        self.get_current_speach()._tick_speach_time()

    # Pseudoprivate
    def _audit(self):
        pass

discussion = Discussion(mover=Speaker('Cici'))
discussion.add_speach('Naman',is_response=True)
discussion.add_speach('Evan',is_response=False)

def main(stdscr):

    # Setup
    #try:
    #    curses.curs_set(False)
    #except curses.error:
    #    pass

    global discussion
    do_hints = True
    prioritize_shy_speakers = True
    
    while(True):

        APP_WIDTH = 50
        APP_X_POS = (curses.COLS-APP_WIDTH)/2
        APP_Y_POS = 10

        SPEAKERS_HEIGHT = 30
        SPEAKERS_Y_POS = 0
        
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
        for y in range(APP_Y_POS, APP_Y_POS+SPEAKERS_HEIGHT):
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
        
        speakers_box = curses.newwin(SPEAKERS_HEIGHT, APP_WIDTH, APP_Y_POS+SPEAKERS_Y_POS, APP_X_POS)
        def update_speakers_box():

            def add_speaker(header, speach, do_newline=True, box=speakers_box, highlight=False):
                speaker_name = speach.get_speaker().get_name().capitalize()
                speaker_time = format_time(speach.get_duration()) + ' / ' + format_time(speach.get_speaker().get_total_speaking_time())
                box.addch(curses.ACS_RARROW if highlight else ' ')
                for ch in header:
                    box.addch(ch)
                box.addstr(speaker_name, curses.A_BOLD if highlight else 0)
                for x in range(len(header) + len(speaker_name) + 2, APP_WIDTH-len(speaker_time)-2):
                    box.addch('.')
                box.addstr(speaker_time)
                box.addch(' ')
                box.addch(curses.ACS_LARROW if highlight else ' ')
                if do_newline:
                    box.addch('\n')

            speakers_box.clear()

            number_of_past_speaches_displayed = min(3,len(discussion.get_past_speaches()))
            
            # This requires a bit of explanation, because it's a little more than just the straight speaking order. I've also allowed for some flags which affect properties of any speach that comes after
            # hlt - turns on highlight
            # nhlt - turn off highlight
            # endl - put a '\n' at the end of the line
            # nendl - don't put a '\n' at the end of the line
            speaking_order = discussion.get_past_speaches()[-3:] + ['hlt', discussion.get_current_speach(), 'nhlt'] + discussion.take_upcoming_speaches(how_many=SPEAKERS_HEIGHT-number_of_past_speaches_displayed-1) # [Speach or 'highlight' or 'nonl']
            speaking_order.insert(-1,'nendl')

            highlight = False
            do_newline = True
            # n = max(0, len(discussion.get_past_speaches())-3)
            for i in range(len(speaking_order)):
                if type(speaking_order[i]) == str:
                    if speaking_order[i] == 'hlt':
                        highlight = True
                    elif speaking_order[i] == 'nhlt':
                        highlight = False
                    elif speaking_order[i] == 'endl':
                        do_newline = True
                    elif speaking_order[i] == 'nendl':
                        do_newline = False
                else:
                    if speaking_order[i].IS_RESPONSE:
                        header = [' ']
                        if False: #i+1 < len(speaking_order) and speaking_order[i+1].IS_RESPONSE:
                            header += [curses.ACS_LTEE]
                        else:
                            header += [curses.ACS_LLCORNER]
                    else:
                        header = ' ' # + str(n) + '. '
                        # n += 1
                    add_speaker(header, speaking_order[i], do_newline=do_newline, highlight=highlight)

            speakers_box.refresh()

        update_speakers_box()
        
        CLOCK_LABEL_TEXT = 'Total Time Elapsed: '
        CLOCK_LABEL_WIDTH = len(CLOCK_LABEL_TEXT)
        clock_label = curses.newwin(CLOCK_HEIGHT, CLOCK_LABEL_WIDTH+1, APP_Y_POS+CLOCK_Y_POS, APP_X_POS)
        clock_label.addstr(CLOCK_LABEL_TEXT, curses.A_BOLD)
        clock_label.refresh()

        PROMPT_TEXT = 'Add Speaker: '
        PROMPT_WIDTH = len(PROMPT_TEXT)
        prompt_win = curses.newwin(PROMPT_HEIGHT, PROMPT_WIDTH+1, APP_Y_POS+PROMPT_Y_POS, APP_X_POS)
        prompt_win.addstr(PROMPT_TEXT)
        prompt_win.refresh()

        #INPUT_CURSOR = '_'
        input_win = curses.newwin(PROMPT_HEIGHT, APP_WIDTH-PROMPT_WIDTH, APP_Y_POS+PROMPT_Y_POS, APP_X_POS+PROMPT_WIDTH)
        #input_win.addch(INPUT_CURSOR)

        subprompt_win = curses.newwin(SUBPROMPT_HEIGHT, APP_WIDTH, APP_Y_POS+SUBPROMPT_Y_POS, APP_X_POS)

        clock_face = curses.newwin(CLOCK_HEIGHT, APP_WIDTH-CLOCK_LABEL_WIDTH, APP_Y_POS+CLOCK_Y_POS, APP_X_POS+CLOCK_LABEL_WIDTH)
        def update_clock():
            clock_face.clear()
            clock_face.addstr(format_time(discussion.get_duration()))
            clock_face.refresh()
            input_win.refresh()
        update_clock()

        input_content = ''
        autocomplete_guess = ''
        def subprompt_autocomplete():
            speaker_guess = discussion.find_speaker(input_content, matching_function=detect_partial_match)
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
            
            current_time =  math.trunc(time.time())
                
            if current_time > most_recent_recorded_time:
                discussion.tick_clock() 
                update_speakers_box()
                update_clock()
                most_recent_recorded_time = current_time

            try:
                key = input_win.getkey()
                ord(key) # Just making sure that this function works, because for some reason it occasionally gets strings of length 10
            except:
                continue
            
            if mode == 0:
                if ord(key) == 127: # Backspace
                    input_content = input_content[:-1]
                    autocomplete_guess = subprompt_autocomplete()
                elif key == ' ' or (ord(key) >= ord('a') and ord(key) <= ord('z')) or (ord(key) >= ord('A') and ord(key) <= ord('Z')) :
                    if len(input_content) < APP_WIDTH-PROMPT_WIDTH-1:
                        input_content += key
                        autocomplete_guess = subprompt_autocomplete()
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
                    do_hints = not do_hints
                    break
                elif ord(key) == 14: # Ctrl-N (Next)
                    discussion.goto_next_speach()
                    update_speakers_box()
                elif ord(key) == 2: # Ctrl-B (Go back to previous speach)
                    discussion.goto_previous_speach()
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
                if key == '1' or key == '2':
                    discussion.add_speach(speaker_name=input_content, is_response=(key=='2'))
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


#discussion = Discussion(mover=Speaker(raw_input('Who moved the motion? ')))
curses.wrapper(main)